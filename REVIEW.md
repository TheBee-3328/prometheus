# P3-REVIEW — SCORE baseline 36483469 (match + preprocessor)

Branch: `agent/p3-review` · Commit under test: `c515919` (main) · Budget: docs + adversarial scripts only.

## Summary

| # | Checklist item | Result |
|---|---|---|
| 1 | Determinism | **PASS** |
| 2 | Round-trip adversarial | **FAIL** |
| 3 | Judge integrity | **PASS** |
| 4 | Limits (enwik8 measure + enwik9 extrapolate) | **PASS** |
| 5 | Code size (contributors + removal suggestions) | **PASS** (analysis) |

**Overall:** FAIL (item 2). enwik8 SCORE line remains valid for contest XML; binary / control-byte inputs do not round-trip.

---

## 1. Determinism — PASS

Inspected `crates/prometheus-core` model path (`model.rs`, `match_model.rs`, `arithmetic.rs`, `preprocessor.rs`) and both CLI mains.

| Risk | Finding |
|---|---|
| Floating point | None (`u8`/`u16`/`u32`/`u64` only; Laplace via integer divide) |
| `HashMap`/`HashSet` iteration | None (arrays / boxed slices; dict is `&[&[u8]]` fixed order) |
| Threads | None |
| Unseeded RNG / time / env | None in model path |
| Pointer values as entropy | None |
| Platform-dependent | LE `u64` length in archive header (documented; Linux x86_64 judge only) |

Judge already enforces compress×2 / decompress×2 bit-identity on enwik8 (`ROUNDTRIP=OK`).

---

## 2. Round-trip adversarial — FAIL

Script: `tests/adversarial_roundtrip.sh`

```bash
nix develop -c ./tests/adversarial_roundtrip.sh
```

| Case | Result |
|---|---|
| empty | PASS |
| one byte (`0x42`) | PASS |
| all zeros (4 KiB) | PASS |
| random 64 KiB | **FAIL** (cmp mismatch; output shorter) |
| invalid UTF-8 | PASS |
| 1 MiB repeated line | PASS |
| enwik8 first 1 / 10 / 100 MB | PASS |

### Reproduction (minimal)

```bash
nix develop -c bash -c '
  cargo build --release -p prometheus-compress -p prometheus-decompress
  printf "\x01" > /tmp/p.bin
  target/release/prometheus-compress /tmp/p.bin /tmp/p.c
  target/release/prometheus-decompress /tmp/p.c /tmp/p.o
  cmp /tmp/p.bin /tmp/p.o || echo FAIL
  # Observed: input 01 → output 20 (space)
'
```

### Root cause

Preprocessor capitalization markers occupy raw bytes `0x01`–`0x08` (`CAP_SPACE` … `CAP_BOS` in `preprocessor.rs`). `encode_capitals` passes those bytes through unchanged; `decode_capitals` treats them as markers and mutates / drops following data.

- enwik8 / sampled enwik9 contain **zero** bytes in `0x01`–`0x08`, so the official corpus still round-trips.
- Existing unit test `encode_decode_empty_and_edge` includes `0x00,0x80,0xFF` but **not** `0x01`–`0x08`.

**Clarification (docs-only):** losslessness is required for arbitrary byte strings, not only enwik XML. Cap markers must not alias plaintext bytes (escape them, or place markers in an already-escaped high-byte space).

---

## 3. Judge integrity — PASS

| Check | Evidence |
|---|---|
| Decompressor never opens original dataset | `prometheus-decompress` opens only CLI `<input>` archive + writes `<output>`; no `enwik*` paths, no side reads |
| Aux files counted in `DECOMPRESSOR` | No `judge_aux.txt`. Dictionary is `static DICT` **embedded in the stripped binary**, so it is already in the 303960 exe size that `judge.sh` measures |
| Corrupt archive | `judge_selftest.sh` still requires failure on bad magic / flipped payload |

```bash
nix develop -c ./judge_selftest.sh
# expect: SELFTEST PASS
```

---

## 4. Limits — PASS

Measured (prior judge on this tree; times from `judge_work/time_*.txt`):

```
ROUNDTRIP=OK COMPRESSED=36179509 DECOMPRESSOR=303960 SCORE=36483469
COMPRESS_SECONDS=30.79 DECOMPRESS_SECONDS=22.12 PEAK_RSS_MB=344
```

| Limit | Spec | enwik8 | enwik9 extrapolate | Verdict |
|---|---|---|---|---|
| Soft wall (dev) | ≤ 30 min each | 31s / 22s | ~5–8 min each if ~10× linear in *n* | OK |
| Soft RSS (dev) | ≤ 4 GB | 344 MB | — | OK |
| Hard RSS | ≤ 10 GB | — | ~2.5–4 GB (model ≈ 80 MB fixed: 16 MiB buf + 8M×u32 ht + order-2 tables; plus ~1 GB decoded + ~0.94× pre buffer) | OK |
| Hard time target | ≤ ~50 h | — | ≪ 1 h | OK |

**Note (not a FAIL):** both CLIs `read_to_end` / allocate full pre + decoded buffers. Fixed-size match tables help, but full-file buffering is the dominant enwik9 RAM term. Streaming would add headroom, not required to pass current limits.

---

## 5. Code size — PASS (analysis)

Stripped `prometheus-decompress`: **303960** bytes (matches judge).

| Contributor | Approx | Notes |
|---|---|---|
| `.text` | ~223 KiB | Dominant; mostly Rust std I/O + alloc + model |
| `.rodata` | ~21 KiB | Includes DICT literals + fmt/panic strings (`strings` ≈ 20 KiB) |
| `.eh_frame` + hdr | ~24 KiB | Unwind tables despite `panic = "abort"` |
| DICT payloads | ~269 B | 35 entries; already tiny vs SCORE |

### Removals that likely save more bytes than they cost in COMPRESSED

1. **Strip unwind / panic fmt** (`-Z build-std=std,panic_abort`, `panic_immediate_abort`, and/or link without eh_frame) — expect tens of KiB off `DECOMPRESSOR` with **zero** model change. Highest ROI.
2. Drop unused `eprintln!` / usage string paths if a `#![no_main]`-lite CLI is acceptable — small (hundreds of B).
3. **Do not** drop short DICT entries (`[[`, `  `, …) for size alone — each is ~2 B in the binary but pays for itself many times on enwik.

Not recommended: moving DICT to an aux file (judge would add those bytes 1:1; no win unless shared somehow).

---

## Prioritized fix list

1. **P0 — preprocessor marker collision**  
   Escape or relocate `0x01`–`0x08` markers so every byte string round-trips. Add adversarial coverage (control bytes + `/dev/urandom`) to `prometheus-core` tests / `tests/adversarial_roundtrip.sh`.  
   *Until fixed, claim lossless only for enwik-like text without those controls.*

2. **P1 — unit-test gap**  
   Extend `encode_decode_empty_and_edge` with `0x01`–`0x08` and a short random blob (would have caught this before merge).

3. **P2 — decompressor size**  
   Panic-abort + no eh_frame / build-std; target cutting `DECOMPRESSOR` without touching predictions (direct SCORE win).

4. **P3 — RAM shape (optional)**  
   Stream encode/decode and avoid holding both pre and decoded full files when approaching enwik9; softens peak RSS risk if model tables grow.

---

## Commands cheat-sheet

```bash
nix develop -c ./judge_selftest.sh
nix develop -c ./judge.sh enwik8 .
nix develop -c ./tests/adversarial_roundtrip.sh
```
