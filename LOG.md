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
| 2026-11-24 | P3-TUNER | params search; merged a61e773; SCORE 28355431 | 28051471 | 303960 | 28355431 | 37.04 | 28.11 | 584 | PASS merge |
| 2026-11-24 | P4-ORDERS-MIX | port orders onto tuned base; A=30912214 B=28809463 both worse than 28355431 | — | — | 28355431 | — | — | — | FAIL ×2 abandon |
| 2026-11-24 | ORCH-P4B | Change approach: decompressor size pass | — | — | — | — | — | — | in progress |
| 2026-11-24 | P4-SIZE | build-std+immediate-abort; merged fa7446b | 28051471 | 28872 | 28080343 | 35.15 | 27.11 | 583 | PASS merge |
| 2026-11-24 | ORCH-WATCH | Document mid-flight resume = platform limit; watchdog timeouts | — | — | — | — | — | — | ok |
| 2026-11-24 | ORCH-HANDOFF | HANDOFF.md+SETUP.md; enwik8×2 + enwik9×20 parts @50MB; reconstruct OK | 28051471 | 28872 | 28080343 | — | — | — | ready |
| 2026-11-24 | ORCH-LIVE | Operator notified; starting unattended monitor + TUNER-R2 | — | — | 28080343 | — | — | — | live |
| 2026-11-24 | ORCH-STOP | Operator cancelled auto-start; killed orch loop + TUNER-R2 spawn | — | — | 28080343 | — | — | — | stopped |

SETUP | machine ready | SCORE=28080343 | unrestricted=yes
| 2026-11-24 | ORCH-LIVE-2 | Operator go; monitoring live; spawn TUNER-R2 | — | — | 28080343 | — | — | — | live |
| 2026-11-24 | TUNER-R2 | Spawning params search seeded from best (SCORE 28080343) | — | — | 28080343 | — | — | — | in progress |
| 2026-11-24 | TUNER-R2 | MIX/RESCALE seed search; merged b8465d4 | 28013837 | 28872 | 28042709 | 27.24 | 19.97 | 583 | PASS merge |
| 2026-11-24 | ORCH-P5 | Spawning P5-SPARSE-XML ensemble-with-match | — | — | 28042709 | — | — | — | in progress |
| 2026-11-24 | ORCH-BUDGET | Operator removed 40-run/8h auto-stop; manual halt ~15h | — | — | 28042709 | — | — | — | override |
| 2026-11-24 | P5-SPARSE-XML | struct CM ensemble w/ match; merged 0f57ac6 | 27981374 | 29752 | 28011126 | 32.48 | 22.46 | 583 | PASS merge |
| 2026-11-24 | ORCH-P5R | Two <0.5% wins → spawn REVIEWER + TUNER-R3 | — | — | 28011126 | — | — | — | in progress |
| 2026-11-24 | P5-REVIEW | REVIEW.md refresh; adv 10/10; merged bc05942 | — | — | 28011126 | — | — | — | PASS merge |
| 2026-11-24 | ORCH-PAUSE | Operator tmux reload; killed compute; R3_r102_only SCORE=28010892 unmerged | 27981140 | 29752 | 28010892 | — | — | — | paused |
| 2026-11-24 | TUNER-R3 | Interrupted; best measured R3_r102_only beats main; commit pending | 27981140 | 29752 | 28010892 | 32.49 | 22.66 | 583 | UNFINISHED pause |
| 2026-11-24 | ORCH-RESUME | Tmux resume; R3_r102_only re-measure FAIL 28013201; spawn DIAG-FIX | 27983449 | 29752 | 28013201 | — | — | — | R3 invalid |
| 2026-11-24 | DIAG-FIX | Spawning diagnose-and-fix (ablation + hypothesis loop) | — | — | 28011126 | — | — | — | in progress |
