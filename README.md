# PROMETHEUS

Lossless compressor aimed at the **[Hutter Prize](https://www.hutter1.net/prize/)** task: compress **`enwik9`** (first \(10^9\) bytes of English Wikipedia XML) under the contest resource limits, and keep a small decompressor.

This repository is an independent research / engineering effort. It is **not** affiliated with or endorsed by the Hutter Prize organizers.

| | |
|--|--|
| **License** | [MIT](LICENSE) (OSI-approved; suitable for Hutter Prize source disclosure) |
| **Copyright** | © 2026 TheBee-3328, Adam Grman (Adam-Grman), and contributors — see [NOTICE](NOTICE) |
| **Language** | Rust (Nix flake toolchain) |
| **Internal SCORE** | `COMPRESSED_BYTES + DECOMPRESSOR_BYTES` (see [JUDGE.md](JUDGE.md); differs from official Hutter \(S\)) |

## Hutter Prize context

Official rules: [hrules.htm](https://www.hutter1.net/prize/hrules.htm). In short:

- Input: `enwik9` (1 000 000 000 bytes).
- Peak RAM ≤ **10 GB**, temp disk ≤ **100 GB**, **no GPU**.
- Each program must finish in **&lt; 70 000 / T** hours, where **T** is the machine’s **Geekbench 5** score.
- Decompression must use **only** the archive (no network / external files at runtime beyond what is counted in the submission).
- Documented source under an **OSI-approved** license is required before prize payout. Contest FAQ preference: Unlicense → **MIT** → Apache → …

This project’s `./judge.sh` measures a **dev SCORE** (archive + stripped decompressor [+ aux]). Official submission size \(S\) also accounts for the compressor (and may double-count the decompressor under the split-comp/decomp relaxation). Always treat [hrules.htm](https://www.hutter1.net/prize/hrules.htm) as authoritative for a prize claim.

## Quick start

```bash
git clone https://github.com/TheBee-3328/prometheus.git
cd prometheus

# Reconstruct enwik8 / enwik9 (wholes are gitignored)
./scripts/reconstruct_data.sh both

# Toolchain + self-test + enwik8 judge
nix develop -c ./judge_selftest.sh
nix develop -c ./judge.sh enwik8 .

# enwik9 is slow (order of hours) — only when you intend a full measure
nix develop -c ./judge.sh enwik9 .
```

More setup detail: [SETUP.md](SETUP.md). Scoring contract: [JUDGE.md](JUDGE.md). Architecture / limits: [SPEC.md](SPEC.md).

### Judge output (success)

One line including `ROUNDTRIP=OK`, sizes, `SCORE`, timings, peak RSS, working `TIME_LIMIT` / `RAM_LIMIT`, and SHA-256 fields. On round-trip failure the judge prints only `ROUNDTRIP=FAIL` (no SCORE).

Always run with a clean `CARGO_TARGET_DIR` (or unset) so another worktree’s target cannot supply a stale binary.

## Repository layout

| Path | Role |
|--|--|
| `crates/prometheus-core` | Models, preprocessor, arithmetic coder |
| `crates/prometheus-compress` / `prometheus-decompress` | CLI binaries |
| `judge.sh` | Authoritative SCORE harness |
| `STATUS.md` / `LOG.md` / `DIAG.md` / `HANDOFF.md` | Orchestrator memory |
| `best.json` | Last locked measured results |
| `generated/` | Embedded tables (e.g. word dict) counted in decompressor size |

## Copyright and licensing

- **Code:** Copyright © 2026 **TheBee-3328**, **Adam Grman (Adam-Grman)**, and contributors. Licensed under the **MIT License** — see [LICENSE](LICENSE) and [NOTICE](NOTICE).
- **Why MIT:** Satisfies the Hutter Prize OSI-source requirement while keeping copyright; permissive enough for other contestants to study and build on (as the contest intends).
- **Data:** `enwik8` / `enwik9` are **not** part of the MIT grant in this repo. Obtain Wikipedia dump–derived files yourself; Wikimedia / dump terms (e.g. CC BY-SA, GFDL) apply to that content.
- **Name “Hutter Prize”:** belongs to the contest; used here only for description.

Contributions are accepted under the same MIT license and copyright model unless a contributor explicitly states otherwise in writing.

## Current best (dev judge)

See `best.json` and `STATUS.md` for the latest **measured** numbers. Do not treat unmeasured extrapolations or failed round-trips as results.

Checkpoint tag example: `CHKP-b5-20769768` (enwik8).

## Disclaimer

No warranty. Contest payment and eligibility are solely under the organizers’ rules and discretion. This software is research code; verify every claim with `./judge.sh` on your machine before any submission.
