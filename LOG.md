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

## 2026-11-25T01:44:26Z Spawn A1-RETUNE + B1-HASH
- After word-dict merge. Baseline SCORE 21365747. No enwik9.

## 2026-11-25T02:08:07Z DIAG-R6 word-SM 100mb case PASS (not full adv suite)
- Script ran only enwik8_first_100mb then echoed PASS=10 — **do not treat as full adversarial**. SCORE 21263097 still provisional.
- B1 full judge + A1 slice in flight.

## 2026-11-25T02:44:54Z Merge B1-HASH v4
- SCORE **20911620** (−454127). Adv 10/10. Peak 3562 MB.

## 2026-11-25T03:51:51Z Subagent follow-ups
- [B1-HASH](8a0477d9-6999-4a45-b570-bd1484a8b4e8): already merged (`06bc109`); main SCORE **20911620**.
- [DIAG-R6](97afab47-bab4-4332-811c-d75fd55ea774): FINAL SCORE 21263097 — **behind main**; do not merge. Parked (word-SM/G5 useful ideas only if rebased on B1+dict).
- A1 cand A: provisional SCORE ~20852107 (−59513); awaiting determinism confirm before merge.

## 2026-11-25T04:17:46Z Merge A1-RETUNE KEEP
- Merged `0118c25` → main. SCORE **20852107** (COMPRESSED=20765227 DECOMP=86880). Δ −59513 vs B1. Adv 10/10. DET=OK.
- Mix: ORDER0=64 MATCH_IND_CROSS=16 INDIRECT=32 ORDER5=72 (B1 ORDER3/4=27 retained).

## 2026-11-25T04:18:57Z Post-A1: spawn B2 + C1
- Adv A1 **10/10**. Merged params + best.json SCORE **20852107**.
- Spawned agent/b2-mixer + agent/c1-tmpl from main. A1 B/C not promoted (weaker slices).

## 2026-11-25T04:44:46Z Target reframed + C1 gate
- Operator aim: ~10–12MB enwik8 / ~95MB enwik9 (not just 20.5e6).
- C1 structure-stats: ABCD **18.56%** cost share, templates **15.36%** (~2.59MB est) — gate PASS. Proceed mine.
- B2: mixer sweeps ongoing; orch directing away from endless 1D grids.

## 2026-11-25T04:50:04Z Hygiene: prune 12 worktrees; unstick C1
- Removed a1/b1/diag*/p5*/tuner* worktrees. Live: main+b2+c1+pre-v2.
- Spawned C1-MINE supersede; B2 cap watcher if slice Δ&gt;−15KB.

## 2026-11-25T05:02Z ORCH resume (operator: continue where left off)
- Cleared ORCH_HALT after duplicate-subagent kill.
- B2 REJECT stands (best 25MB slice Δ −3909 ≪ −15KB cap). Do not relaunch B2 mixer grids.
- C1: WIP `c86e0e5` templates ON; roundtrip OK; spawning cycle-0 enwik8 judge.
- Next parallel after C1 judge starts: B3 contexts / match / deeper APM (not more SSE2 1D).
- Locked best still SCORE **20852107**. Aim ~10–12MB enwik8.

## 2026-11-25T06:02Z Agents cleared → respawn C1 + B3
- Operator cleared all subagents (0 running).
- Spawned **C1-MINE**: cycle-0 enwik8 judge on `agent/c1-tmpl` (`c86e0e5`, templates ON).
- Spawned **B3-CTX**: new worktree `prometheus-b3-ctx` / `agent/b3-ctx` from main — contexts/match/deeper APM (B2 parked REJECT).
- B2 stays REJECT; no SSE2 grids.

## 2026-11-25T06:28Z C1 cycle-0 judge (templates ON)
```
ROUNDTRIP=FAIL COMPRESSED=20054334 DECOMPRESSOR=91080 SCORE=20145414
COMPRESS_SECONDS=396.10 DECOMPRESS_SECONDS=361.08 PEAK_RSS_MB=3554
```
- COMPRESSED Δ vs 20765227: **−710893** (attractive if RT fixed)
- Decode emits 96256231 B (not 1e8); rt==rt2 (deterministic bug). compress×2 OK.
- No KEEP. C1 must fix encode/decode before SCORE counts.
- B3 RESUME (was paused for RAM).


## 2026-11-25T06:38Z Orch directive reload — state
- Wrote CONSTRAINTS into STATUS.md; tagged `CHKP-a1-20852107` → `0118c25`.
- Track 0B coder audit appended to DIAG (16-bit p confirmed; no SCORE claim).
- C1: root-cause — `find_lit_at_depth` can match literals inside nested later templates; fixing depth + u16 param cap.
- B3: h2 Δ−3774 best; h3 match-SM REJECT (+114092); h4 APM2 REJECT (+6679). Still under −15KB gate.


## 2026-11-25T06:27Z Orch-verified C1 cycle-1 (depth-fix templates ON)
```
ROUNDTRIP=OK COMPRESSED=20777571 DECOMPRESSOR=91080 SCORE=20868651
COMPRESS_SECONDS=407.57 DECOMPRESS_SECONDS=371.23 PEAK_RSS_MB=3563
```
- **Confirmed** by orch-owned `judge.sh` (agent’s prior run hit determinism/strip race — discarded).
- Δ vs best 20852107: **+16544** → not KEEP.
- Cycle-0 FAIL archive (20054334) was invalid encode; ignore for SCORE.
- B3 REJECT confirmed: best 25MiB Δ −3774 (`37da1f6`).


## 2026-11-25T08:34Z Monitoring resumed (operator)
- C1 cycle-3: `SCORE=20866251` (+14144) N=32 — best OK prune so far, still worse than main.
- C1 cycle-4: N=16 judge started.
- B2/B3 stay REJECT. Locked best unchanged 20852107.


## 2026-11-25T08:42Z C1 duplicate judges killed
- Two cycle-4 judges overlapped (compress+decompress); available RAM ~2.5Gi.
- Orch killed races; aligned TEMPLATE_DICT_N=16 with 666 B blob; single orch judge started (`c1_judge_c4_orch.log`).


## 2026-11-25T09:10Z C1 prune trail (monitoring)
| Cycle | Config | SCORE | Δ vs 20852107 |
|------:|--------|------:|--------------:|
| 1 | N=64 | 20868651 | +16544 |
| 2 | INLINE | 20874013 | +21906 |
| 3 | N=32 | 20866251 | +14144 |
| 4 | N=16 | **20863101** | **+10994** (best OK; RT OK; timing file raced) |
| 5 | N=8 | in flight | — |
Prune monotonically helps but still above main. Likely THROW soon.


## 2026-11-25T09:33Z C1 cycle-5
`ROUNDTRIP=OK COMPRESSED=20772849 DECOMPRESSOR=88776 SCORE=20861625` (+9518). Cycle-6 started.


## 2026-11-25T09:59Z C1 cycle-6 N=4
`SCORE=20859637` (+7530). Cycle-7 N=0 (empty blob) started — last prune before THROW.


## 2026-11-25T10:49Z C1 cycle-7 curated
`SCORE=20863631` (+11524) — worse than N=4. Cycle-8 N=2 started; orch will THROW after if still ≥ baseline.


## 2026-11-25T11:16Z C1 TEMPLATE-MINER THROW
- Agent tip `3609879`: `PRE_STAGE_TEMPLATES=false`. Not merged (never on main).
- Best OK trial: cycle-8 N=2 `SCORE=20859322` (+7215 vs 20852107).
- Trail: N=64 +16544 → N=4 +7530 → N=2 +7215; curated worse.
- Next: hierarchical / adaptive mix (B2/B3 style 1D rejected).


## 2026-11-25T11:17Z Spawn B4-HMIX
- Worktree `prometheus-b4-hmix` / `agent/b4-hmix` from main.
- Brief: hierarchical / adaptive mix; 25MiB gate −15KB; no B2/B3/C1 repeats.


## 2026-11-25T11:40Z B4-HMIX REJECT
- Tip `66e0b41`. Best 25MiB Δ **−1280** (H2c match-on IND/CROSS mute). Gate −15KB.
- Two-layer / dual-rate / adaptive group all regress. No full judge.
- Next: Track 0 refresh on current main best (bit-cost by class) before new feature lane.


## 2026-11-25T11:41Z Spawn T0-REFRESH
- After B4 REJECT. Worktree `prometheus-t0` / `agent/t0-refresh`.
- Re-measure bit costs on current best; rank next lane. No SCORE chase unless coder bug.


## 2026-11-25T12:00Z T0-REFRESH complete (`5ddadc4`)
- Diag archive COMPRESSED **20765227** matches locked best.
- Struct: other 81.44%, template 15.36% (C1 THROW stands), ABCD 18.56%.
- Byte-class: alpha 35.84%, ws 23.97%, other 22.89%, hibyte 13.33%, digit 3.57%.
- Coder: 16-bit p, zero pin clamps, SSE→SSE2→APM; no free coder hyp.
- Rank #1 next: residual NL/prose (alpha+other). Then whitespace, hibyte/token.


## 2026-11-25T12:01Z Spawn B5-PROSE
- After T0 rank #1 (alpha/other residual). Worktree `prometheus-b5-prose` / `agent/b5-prose`.
- Gate: 25MiB Δ ≤ −15KB then full judge.


## 2026-11-25T13:03Z Merge B5-PROSE KEEP
- Merged `agent/b5-prose` (`0c3248c`) → main.
- SCORE **20769768** (COMPRESSED=20681576 DECOMP=88192). Δ vs A1 **−82339**.
- Adv **10/10 PASS**. Judge line from `tune/b5_full_judge.log`.
- Word hit-gate + prose prev+cur (MODE=2 MIX=24 BITS=25).


## 2026-11-25T13:04Z Spawn B6-WS
- After B5 merge. Whitespace model (T0 rank #2, ≲5MB cap). Baseline now SCORE 20769768.

| 2026-11-25 | B6-WS | WS CM modes/mix/bits/gate/post; best slice Δ −3133 (gate −15000) | — | — | — | — | — | — | REJECT |
| 2026-11-25 | PHASE2 | §2: decomp mismatch=CARGO_TARGET_DIR→c1 stale 37k; official time=70000/T (T=?); working 4h+9GB in judge; webhook \\n fix; enwik9 B5 re-run parallel | — | — | — | — | — | — | in progress |
