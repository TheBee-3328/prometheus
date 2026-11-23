# LOG

Append-only. Every agent result (success or failure) goes here.

| Date (UTC) | Task | Tried | COMPRESSED | DECOMP | SCORE | COMP_s | DECOMP_s | RSS_MB | Verdict |
|---|---|---|---|---|---|---|---|---|---|
| 2026-11-23 | ORCH-INIT | Created SPEC.md, STATUS.md, TASK_TEMPLATE.md, LOG.md; extracted enwik8; spawning P0-JUDGE | — | — | — | — | — | — | in progress |
| 2026-11-23 | ORCH-NIX | Added flake.nix + flake.lock; SPEC requires `nix develop`; verified rustc/cargo/strip/cmp/time in shell; branch agent/p0-judge | — | — | — | — | — | — | ok |
| 2026-11-23 | P0-JUDGE | judge.sh + selftest + store baseline; merged to main @ 8d6ba5d | 100000012 | 299864 | 100299876 | 0.14 | 0.13 | 98 | PASS |
| 2026-11-23 | ORCH-P1 | Spawning P1-BUILD: traits, arithmetic coder, order-2 model | — | — | — | — | — | — | in progress |

## Notes

- Official rules recorded in SPEC.md from https://www.hutter1.net/prize/hrules.htm
- Internal SCORE = compressed + decompressor (+ aux files). Never claim enwik9 results without `./judge.sh enwik9`.
| 2026-11-23 | P1-BUILD | order-2 + arithmetic coder; merged d170a6a | 37955033 | 299864 | 38254897 | 12.11 | 13.33 | 113 | PASS |
| 2026-11-23 | ORCH-P2 | Spawning 5 parallel builders (orders/match/word/mixer/pre) | — | — | — | — | — | — | in progress |
| 2026-11-24 | P2-MATCH | LZ match + order-2; merged c5a34d9 | 36994755 | 303960 | 37298715 | 23.74 | 25.70 | 161 | PASS merge |
| 2026-11-24 | P2-PRE | caps+XML dict preprocessor; beat P1 but worse than MATCH | 37170074 | 299864 | 37469938 | 24.11 | 14.10 | 297 | PASS hold |
| 2026-11-24 | P2-MIXER | two-layer mixer+APM orders 0/1/2 | 37588267 | 312152 | 37900419 | 86.31 | 86.35 | 113 | PASS hold (worse than MATCH) |
| 2026-11-24 | P2-WORD | word+order-2 | 38702992 | 303960 | 39006952 | 93.32 | 82.77 | 128 | FAIL (worse than P1) |
| 2026-11-24 | P2-ORDERS | hashed orders 0–6 CM | 37640963 | 303960 | 37944923 | 131.55 | 127.86 | 137 | PASS hold (worse than MATCH) |
| 2026-11-24 | ORCH-P2I | Spawning integrate MATCH+PRE | — | — | — | — | — | — | in progress |
| 2026-11-24 | P2-INTEGRATE | match + preprocessor; merged c44bb4b | 36179509 | 303960 | 36483469 | 30.79 | 22.12 | 344 | PASS merge |
| 2026-11-24 | ORCH-P3 | Spawning P3-REVIEW | — | — | — | — | — | — | in progress |
| 2026-11-24 | P3-REVIEW | adversarial FAIL: pre markers 0x01-0x08; REVIEW.md merged | — | — | 36483469 | — | — | — | FAIL item2 |
| 2026-11-24 | ORCH-P3F | Spawning P3-FIX-PRE | — | — | — | — | — | — | in progress |
| 2026-11-24 | P3-FIX-PRE | escape 0x01-0x08/0x7F; adv 10/10; merged 9c4fd3c | 36179509 | 303960 | 36483469 | 32.87 | 23.31 | 344 | PASS merge |
| 2026-11-24 | ORCH-P3T | Spawning P3-TUNER | — | — | — | — | — | — | in progress |
