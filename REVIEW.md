# P5-REVIEW — SCORE baseline 28011126 (post P5-SPARSE-XML)

Branch: `agent/p5-review` · Commit under test: `c39087f` / merge `86ba16c` (main) · Scope: docs + adversarial verification only (no SCORE chasing).

## Summary

| # | Checklist item | Result |
|---|---|---|
| 1 | Determinism | **PASS** |
| 2 | Round-trip adversarial (10/10) | **PASS** |
| 3 | Judge integrity | **PASS** |
| 4 | Limits (enwik8 measure + enwik9 extrapolate) | **PASS** |
| 5 | Code size (contributors + removal suggestions) | **PASS** (analysis) |

**Overall:** **PASS** (all five). Baseline line remains valid for merge/tuning:

```
ROUNDTRIP=OK COMPRESSED=27981374 DECOMPRESSOR=29752 SCORE=28011126
COMPRESS_SECONDS=32.48 DECOMPRESS_SECONDS=22.46 PEAK_RSS_MB=583
```

Δ vs TUNER-R2 (28042709): −31583 (−0.11%). Prior P3-REVIEW adversarial FAIL (cap-marker collision) is resolved via `CAP_ESC` (`0x7F`) in the preprocessor.

---

## 1. Determinism — PASS

Inspected model path after sparse merge: `model.rs`, `match_model.rs`, `sparse.rs`, `arithmetic.rs`, `preprocessor.rs`, `params.rs`, both CLI mains.

| Risk | Finding |
|---|---|
| Floating point | None (`u8`/`u16`/`u32`/`u64` only; Laplace / mix via integer divide) |
| `HashMap`/`HashSet` iteration | None (arrays / boxed slices; dict is `&[&[u8]]` fixed order) |
| Threads | None |
| Unseeded RNG / time / env | None in model path (`std::env` only for CLI argv) |
| Pointer values as entropy | None |
| Sparse/XML struct CM | Integer counters + fixed state machine; ensembled only when match inactive (`STRUCT_MIX_WEIGHT`) |
| Platform-dependent | LE `u64` length in archive header (documented; Linux x86_64 judge only) |

Adversarial suite re-checks compress×2 bit-identity per case. Judge enforces compress×2 / decompress×2 on enwik8 (`ROUNDTRIP=OK`).

---

## 2. Round-trip adversarial — PASS (10/10)

```bash
nix develop -c ./tests/adversarial_roundtrip.sh
# ADVERSARIAL PASS=10 FAIL=0
```

| Case | Result |
|---|---|
| empty | PASS |
| one byte (`0x42`) | PASS |
| all zeros (4 KiB) | PASS |
| ctrl_markers (`0x01`–`0x08` + `0x7F` + C0 controls) | PASS |
| random 64 KiB | PASS |
| invalid UTF-8 | PASS |
| 1 MiB repeated line | PASS |
| enwik8 first 1 / 10 / 100 MB | PASS |

No script or algorithm changes required this round. Cap-marker escape remains covered by case 4 + unit tests in `preprocessor.rs`.

---

## 3. Judge integrity — PASS

| Check | Evidence |
|---|---|
| Decompressor never opens original dataset | `prometheus-decompress` opens only CLI `<input>` archive + writes `<output>`; no `enwik*` paths, no side reads |
| Aux files counted in `DECOMPRESSOR` | No `judge_aux.txt`. Dictionary / tables are embedded or heap-allocated; stripped exe size **29752** matches recorded SCORE |
| Corrupt archive | `judge_selftest.sh` → `SELFTEST PASS` (bad magic / flipped payload must fail) |

```bash
nix develop -c ./judge_selftest.sh
# SELFTEST PASS
```

---

## 4. Limits — PASS

Measured (P5-SPARSE-XML merge judge, logged):

```
ROUNDTRIP=OK COMPRESSED=27981374 DECOMPRESSOR=29752 SCORE=28011126
COMPRESS_SECONDS=32.48 DECOMPRESS_SECONDS=22.46 PEAK_RSS_MB=583
```

| Limit | Spec | enwik8 | enwik9 extrapolate | Verdict |
|---|---|---|---|---|
| Soft wall (dev) | ≤ 30 min each | ~32 s / ~22 s | ~5–8 min each if ~10× linear in *n* | OK |
| Soft RSS (dev) | ≤ 4 GB | 583 MB | — | OK |
| Hard RSS | ≤ 10 GB | — | ~2.5–4 GB (match buf 32 MiB + hash ~256 MiB u32 + order-2 + sparse ~36 KiB counters; plus full-file buffers) | OK |
| Hard time target | ≤ ~50 h | — | ≪ 1 h | OK |

**Note (not a FAIL):** both CLIs still `read_to_end` / hold full pre + decoded buffers. Sparse tables are tiny vs match HT; full-file buffering remains the dominant enwik9 RAM term. Streaming would add headroom only.

---

## 5. Code size — PASS (analysis)

Stripped `prometheus-decompress`: **29752** bytes (matches judge; +880 vs TUNER-R2 `28872` after sparse `.text`).

`size` (symbols stripped): text≈24821 data≈1744 bss≈1488.

| Contributor | Approx | Notes |
|---|---|---|
| Match + order-2 + arithmetic | dominant `.text` | Core path; already size-optimized (`build-std`, `panic=immediate-abort`, no unwind tables) |
| Sparse/XML (`sparse.rs`) | ~+880 B exe | Struct-state CM; heap tables (~18k×2 counters) not in SCORE |
| Preprocessor DICT | small `.rodata` | Embedded; already in exe size |
| Params / mix knobs | tiny | Compile-time constants only |

### Removals / size notes (do not chase SCORE here)

1. Further panic/fmt stripping is largely done; residual gains are small vs model knobs.
2. Do **not** drop `STRUCT_MIX_WEIGHT` path for size alone — it paid −31k COMPRESSED for +880 DECOMPRESSOR.
3. Do not move DICT/tables to aux files (judge adds aux 1:1).

---

## Prioritized notes for TUNER-R3

1. **No P0 round-trip blockers.** Safe to tune `params.rs` only (`ORDER2_RESCALE`, `MIX_WEIGHT`, `MATCH_CONF`, `STRUCT_MIX_WEIGHT`, hash/buf bits if in scope).
2. **Seed from current best:** `STRUCT_MIX_WEIGHT=14`, `ORDER2_RESCALE=100`, `MIX_WEIGHT=[120,160,190,230,245]`, max match conf.
3. **Watch DECOMPRESSOR:** param-only edits should keep exe size flat; any new code path needs SCORE math vs compressed gain.
4. **Optional later (not TUNER):** streaming I/O for enwik9 RSS headroom; not required for current limits.

---

## Commands cheat-sheet

```bash
nix develop -c ./judge_selftest.sh
nix develop -c ./judge.sh enwik8 .
nix develop -c ./tests/adversarial_roundtrip.sh
```
