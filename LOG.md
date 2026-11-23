# LOG

Append-only. Every agent result (success or failure) goes here.

| Date (UTC) | Task | Tried | COMPRESSED | DECOMP | SCORE | COMP_s | DECOMP_s | RSS_MB | Verdict |
|---|---|---|---|---|---|---|---|---|---|
| 2026-11-23 | ORCH-INIT | Created SPEC.md, STATUS.md, TASK_TEMPLATE.md, LOG.md; extracted enwik8; spawning P0-JUDGE | — | — | — | — | — | — | in progress |
| 2026-11-23 | ORCH-NIX | Added flake.nix + flake.lock; SPEC requires `nix develop`; verified rustc/cargo/strip/cmp/time in shell; branch agent/p0-judge | — | — | — | — | — | — | ok |

## Notes

- Official rules recorded in SPEC.md from https://www.hutter1.net/prize/hrules.htm
- Internal SCORE = compressed + decompressor (+ aux files). Never claim enwik9 results without `./judge.sh enwik9`.
