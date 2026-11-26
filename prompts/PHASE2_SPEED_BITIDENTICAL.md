# Agent brief — Phase 2 Step 1: bit-identical speedups (B5 tip)

**Branch / worktree:** create `agent/p2-speed` from `main` @ `CHKP-b5-20769768` (or current main after enwik9 bookkeeping).  
**CARGO_TARGET_DIR:** worktree-local only (never poison main / other trees).  
**Nix:** always `nix develop -c …`.

## Goal
Cut wall/CPU on compress+decompress **without changing one archive bit** vs B5 tip. Every second saved is budget for bigger models (Phase 2 budget rule).

## Baseline (must match exactly)
- enwik8 SCORE **20769768** (or re-measure after clean `env -u CARGO_TARGET_DIR nix develop -c ./judge.sh enwik8 .`).
- 25 MiB tip: **5680631** (`tune/slice25.bin`).
- Acceptance: `cmp`/`sha256` of archive identical to tip on `slice25` **and** full enwik8 before KEEP.

## Allowed edits
- Cache-friendlier hash / table layout, prefetch, deterministic SIMD (`target-cpu` already native in flake — no nondeterministic reduction order).
- Hot-path microopts in match / StateMap / mixer **that do not change p or adapt updates**.
- Build flags that do not change numeric results.

## Forbidden
- Changing mix weights, learning rates, table sizes, or prediction math “for speed.”
- Threads in model path; FP in model path.
- Editing `judge.sh` / scoring.
- Claiming SCORE wins from speed alone (speed is budget, not SCORE).

## Measure / report
| Metric | Before | After |
|--|--:|--:|
| slice25 wall_s / maxRSS | | |
| enwik8 COMPRESS_SECONDS / DECOMPRESS_SECONDS / PEAK_RSS | | |
| archive sha256 | must match tip | |

Budget line: **CPU-hours saved** (enwik8 wall Δ); projected enwik9 save ≈ ×10–12.

## Done when
Bit-identical tip archives + documented CPU save, or THROW with numbers after ≤2 serious attempts.
