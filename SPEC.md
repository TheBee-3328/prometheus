# PROMETHEUS — Specification & Contract

**Goal:** Lossless compressor for `enwik9` (first 10^9 bytes of English Wikipedia XML) minimizing **SCORE = compressed_file_bytes + decompressor_bytes** (Hutter Prize–style). Never claim a result the judge has not measured.

**Realistic targets:** 6×–7× first; ~9× stretch. Current official Hutter record (as of late 2026): ~100 424 672 total under their S formula — our internal SCORE is the simpler sum above.

**Overall budget:** **No auto-stop.** Operator will halt the orch manually (target window ~15 h from 2026-11-24 ~03:33 UTC+1). Still log agent-run counts in `STATUS.md` for visibility; do **not** stop spawning or write end-of-budget `REPORT.md` solely because of old 40-run / 8-hour caps. Soft failures and watchdog timeouts still apply per task.

**Nix (mandatory):** This machine is NixOS/Nix. All builds, tests, and judge runs must use the project flake:

```bash
nix develop -c ./judge.sh enwik8 .
# or: nix develop   # then run commands inside the shell
```

Do not assume host `~/.cargo` toolchains or system packages beyond what `flake.nix` provides. Prefer `nix develop -c <cmd>` in scripts and agent briefs. `judge.sh` itself should invoke cargo/strip/time from PATH after entering the flake shell (orchestrator/agents enter nix before calling judge, or judge re-execs under `nix develop` if `IN_NIX_SHELL` / flake marker is unset).

---

## Official Hutter Prize constraints (source of truth)

Retrieved from https://www.hutter1.net/prize/hrules.htm (and prize page summary):

| Constraint | Exact value |
|---|---|
| Input | `enwik9` = 1 000 000 000 bytes |
| Peak RAM | ≤ **10 GB** |
| Temporary disk | ≤ **100 GB** |
| Runtime | < **70 000 / T** hours per program, `T` = Geekbench 5 score of the machine; no GPU |
| Network / external data at decompress | **Forbidden** — must reproduce from archive alone |
| Separate compressor + decompressor scoring (relaxation) | `S := length(comp/zip) + 2×length(decomp/zip) + length(archive)`; if comp≡decomp, 2× → 1× |
| Self-extracting archive form | `S := length(comp/zip) + length(archive.exe)` |

**This project's internal judge SCORE** (orchestrator contract, not official S):

```
SCORE = COMPRESSED_BYTES + DECOMPRESSOR_BYTES
```

where `DECOMPRESSOR_BYTES` = stripped decompressor executable + every auxiliary file it reads at runtime (dictionaries, weights). Documented in `JUDGE.md`. Compressor size is **not** added to our SCORE (dev convenience); official submission would need to account for it.

**Dev time/RAM soft limits** (enwik8): compression ≤ 30 min, decompression ≤ 30 min, peak RSS ≤ 4 GB.  
**enwik9 hard limits** (must stay under): peak RSS ≤ 10 GB; wall time target ≤ 50 h single-core equivalent on a ~Geekbench5≈1300–1400 machine (align with contest machines).

---

## Architecture contract

Core language: **Rust**, single **Cargo workspace**. Integer / fixed-point math in the model path so compressor and decompressor are bit-identical. No threads in the model path unless proven deterministic. No floating point in the model path. No `HashMap`/`HashSet` iteration-order dependence.

### 1. Preprocessor

Reversible transform (XML structure split, capitalization flags, optional dictionary).

```rust
fn encode(input: &[u8]) -> Vec<u8>;
fn decode(input: &[u8]) -> Vec<u8>;
```

### 2. Model

```rust
trait Model {
    fn predict(&mut self) -> u16; // p(bit=1) in 0..=65535 scale
    fn update(&mut self, bit: u8);
}
```

Built from context models: orders 1–6, word model, sparse contexts, XML-state context, match model.

### 3. Mixer

Logistic mixing with selectable weight sets, two layers, then APM/SSE chain. Optional LSTM later.

### 4. Arithmetic coder

32-bit binary arithmetic coder with 16-bit probabilities.

### 5. Decompressor

Tiny binary that runs the same model in lockstep; must reproduce `enwik9` byte-exactly from the compressed file alone.

---

## CLI contract (baseline / production)

Workspace binaries (names fixed for the judge):

- `prometheus-compress <input> <output>`
- `prometheus-decompress <input> <output>`

---

## Phases

| Phase | Focus |
|---|---|
| 0 | Judge harness + trivial baseline |
| 1 | Workspace skeleton, traits, arithmetic coder, order-2 model |
| 2 | Parallel: orders 1–6, match, word, mixer+APM, preprocessor v1 |
| 3 | Reviewer + tuner on enwik8 |
| 4 | XML/wiki contexts, dictionary, mixer improvements, size/speed |
| 5 | Full enwik9 verification |

Develop and compare on **enwik8**. Confirm on **enwik9** only at phase ends and before declaring milestones.

---

## Merge rule

Merge only if `./judge.sh` reports: `ROUNDTRIP=OK`, SCORE improved (or equal with smaller code), time and RAM within limits.
