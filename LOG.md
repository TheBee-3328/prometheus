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
| 2026-11-24 | DIAG-FIX | o0–o6+word+SSE; merged c3404fa @ 4327fea | 23036149 | 33016 | 23069165 | 192.52 | 185.69 | 888 | PASS merge |
| 2026-11-24 | ORCH-NEXT | Spawn DIAG-R2 mixer/APM toward ≤20.5e6 | — | — | 23069165 | — | — | — | in progress |
| 2026-11-24 | DIAG-R2 | paq-style APM; merged 669042b | 22766317 | 42552 | 22808869 | 206.63 | 201.26 | 888 | PASS merge |
| 2026-11-24 | DIAG-R2 | richer APM cxt; merged e18070b | 22567638 | 42568 | 22610206 | 217.53 | 207.27 | 889 | PASS merge |
| 2026-11-24 | DIAG-R2 | hashed order-7 mix=16; merged 35d2a63 | 22531741 | 42760 | 22574501 | 260.00 | 256.14 | 905 | PASS merge |
| 2026-11-24 | DIAG-R2 | FINAL REPORT partial vs 20.5e6; merged 28a76a1 | 22531741 | 42760 | 22574501 | — | — | — | done |
| 2026-11-24 | ORCH-NEXT | Spawn DIAG-R3 multi-match / indirect / state-map | — | — | 22574501 | — | — | — | in progress |
| 2026-11-24 | DIAG-R3 | multi-candidate match×4; merged cda121b | 22439085 | 43352 | 22482437 | 281.75 | 273.51 | 1673 | PASS merge |
| 2026-11-24 | DIAG-R3 | indirect/record MIX=40; merged 6eecaa2 | 22037144 | 44168 | 22081312 | 320.73 | 317.10 | 1705 | PASS merge |
| 2026-11-24 | DIAG-R3 | STRETCH heap LUT; merged 5f645c4 | 22039256 | 36680 | 22075936 | 321.50 | 311.63 | 1705 | PASS merge |
| 2026-11-24 | DIAG-R3 | FINAL REPORT partial; merged 3e70cc7 | 22039256 | 36680 | 22075936 | — | — | — | done |
| 2026-11-24 | ORCH-NEXT | Spawn DIAG-R4 state-map / match×indirect | — | — | 22075936 | — | — | — | in progress |
| 2026-11-24 | DIAG-R4 | Order-2 StateMap; merged 3163073 | 22008866 | 36744 | 22045610 | 315.71 | 329.03 | 1756 | PASS merge |
| 2026-11-24 | DIAG-R4 | indirect order-3; merged e405da9 | 21933188 | 36952 | 21970140 | 342.13 | 354.65 | 1756 | PASS merge |
| 2026-11-24 | DIAG-R4 | match×indirect CROSS=16; merged 896dd9d | 21918203 | 37288 | 21955491 | 346.11 | 341.16 | 1756 | PASS merge |
| 2026-11-24 | DIAG-R4 | StateMap soft-reset; merged 589cfe1 | 21913905 | 37304 | 21951209 | 343.83 | 333.06 | 1756 | PASS merge |
| 2026-11-24 | DIAG-R4 | StateMap Order-3; merged 15af354 | 21871189 | 37544 | 21908733 | 344.23 | 332.04 | 1788 | PASS merge |
| 2026-11-24 | DIAG-R4 | StateMap Order-4; merged 94c24eb | 21849655 | 37512 | 21887167 | 373.35 | 345.21 | 1852 | PASS merge |
| 2026-11-24 | DIAG-R4 | FINAL REPORT partial; merged 99a0a66 | 21849655 | 37512 | 21887167 | — | — | — | done |
| 2026-11-24 | ORCH-NEXT | Spawn DIAG-R5 StateMap o6/o7 / alt-match | — | — | 21887167 | — | — | — | in progress |

## 2026-11-24T21:59:53Z Merge DIAG-R5
- Merged origin/agent/diag-r5 → main. SCORE **21778410** (COMPRESSED=21740866 DECOMP=37544).
- Δ vs R4: SCORE −108757. Gap to 20.5e6 ≈1.24 MB.
- enwik9 judge still finishing on pre-R5 binary (~186.5 MB compressed first pass).

## 2026-11-24T22:00:17Z Spawn DIAG-R6
- Worktree `prometheus-diag-r6` / branch `agent/diag-r6` from main@76284a3.
- Brief: soft-reset schedule, mixer retune, word/o5 hybrid, RSS trim. Baseline SCORE 21778410.

## 2026-11-24T22:51:36Z enwik9 judge (pre-R5 binary)
- `ROUNDTRIP=OK COMPRESSED=186549457 DECOMPRESSOR=37512 SCORE=186586969` (~57+57 min).
- ≈1.492 bpB; COMPRESSED under ~205e6 floor.

## 2026-11-24T22:51:36Z DIAG-R6 asymptotic eq8 judge
- `ROUNDTRIP=OK COMPRESSED=21546316 DECOMPRESSOR=37560 SCORE=21583876` (−194534 vs main 21778410).
- Uncommitted; wait adversarial + commit before merge.

## 2026-11-24T23:12:47Z Operator pause requested
- Pause after DIAG-R6 finishes. No new spawns. See PAUSE.md.
- Told R6 to commit/push eq8 keep (SCORE 21583876, adv 10/10) then stop.

## 2026-11-24T23:15:41Z Merge DIAG-R6 eq8
- Merged `73ab3c8` → main. SCORE **21583876** (−194534 vs R5).
- Gap to 20.5e6 ≈1.05 MB. Operator pause after agent exits.

## 2026-11-24T23:57:16Z Unpause — PRE-V2 dual trials
- Operator: run word-dict then TEMPLATE-MINER; continue after THROW.
- Baseline SCORE **21583876**. Worktree `agent/pre-v2`.

## 2026-11-25T01:01:38Z REVERT mixer G5 merge
- Judge SCORE 21395251 but adversarial **7/10 FAIL** — reverted from main. Parked on agent/diag-r6.

## 2026-11-25T01:03:29Z DIAG-R6 word StateMap slice (parked)
- 20M **−28008** vs G5 (4410359). No full judge until PRE-V2 free.

## 2026-11-25T01:43:09Z Step 1 — baseline locked
- Stopped all agents/orphans (compress/decompress/adversarial/diag-r6/pre-v2 workers).
- Poll cadence set to **2 minutes** (operator).
- Locked enwik8 (verified words-on): SCORE **21365747** COMPRESSED=21279587 DECOMPRESSOR=86160 bpB≈1.702 ROUNDTRIP=OK peak=1834MB times 307.98/290.25s. Adv 10/10.
- main tip still 21583876 until merge.
- enwik9 manual, may be stale: SCORE 186586969 (pre-R5 binary).
- Provisional R6 word-SM SCORE 21263097 ROUNDTRIP=OK — adv interrupted; not locked.
- Written `best.json`. Proceeding Step 2. **No enwik9 auto-runs.**

## 2026-11-25T01:43:54Z Merge PRE-V2 word dict KEEP ([PRE-V2-WORD](438dad27-85ac-482e-ab92-791168648c7f))
- Merged `b25a183` → main. SCORE **21365747** (COMPRESSED=21279587 DECOMP=86160). Adv 10/10. Cycles 0/15.
- N=4096 blob 47482 B. A3 complete. Next: A1 retune + B parallel + C1 analysis.
