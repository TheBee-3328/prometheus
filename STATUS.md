# STATUS — PROMETHEUS Phase 2

**LIVE** 2026-11-27T06:34Z · orch **IMPROVING** · CM4 KEEP landing
**Host CPU:** AMD Ryzen 5 5600X (6C/12T)  
**Geekbench5 T:** **1650 assumed** (not measured yet — GB5 run queued) → H≈42.42 h/dir · working 0.8×H≈33.94 h/dir (122182 s)  
**Best:** enwik8 **20163922** · enwik9 **173961591** — both **UNVERIFIED FOR 10 GB** (no ceiling test yet; prior 9 GB working gate retired)  
**Limits:** `PROMETHEUS_PHASE2_LIMITS.md` — time is the budget; default RAM=NO_CAP  
**License:** GPLv3-or-later

## Active
| Lane | Branch | State |
|--|--|--|
| P2-SPEED | `agent/p2-speed` | **KEEP** on main (−24% wall, bit-id) |
| P2-MATCH | `agent/p2-match` | **THROW** `796353a` (superseded by MATCH2) |
| P2-PRE | `agent/p2-pre` | **KEEP merged** (nocaps; tip superseded) |
| P2-CM | `agent/p2-cm` | **KEEP merged** SCORE=**20686092** |
| P2-CM2 | `agent/p2-cm2` | **KEEP** SCORE=**20566057** · adv 10/10 · origin pushed |
| P2-MATCH2 | `agent/p2-match2` | **THROW** best −3693; closed |
| P2-CM3 | `agent/p2-cm3` | **THROW** 20580794 ≥ 20566057 (agent mislabeled KEEP vs stale tip) |
| P2-O6 | `agent/p2-o6` | **KEEP** SCORE=**20415163** · adv 10/10 · origin pushed |
| P2-SSE2 | `agent/p2-sse2` | **THROW** best Δ=−9014; closed |
| P2-IND2 | `agent/p2-ind2` | **KEEP** SCORE=**20353480** · adv **10/10 PASS** · origin pushed |
| P2-MIX3 | `agent/p2-mix3` | **KEEP** SCORE=**20268625** · origin push via orch_keep_land |
| P2-CM4 | `agent/p2-cm4` | **KEEP** SCORE=**20163922** |

| enwik9 tip | IND2 binary | ACTIVE compress 1CPU/10GB RSS watch → /tmp/p2-enwik9-tip |

## Queue
After CM2/MATCH2 land or THROW → next CM/mixer/indirect family. Neural blocked until §2.4 determinism PASS.

## Orch
Continuous 3m monitor + spawn loop. Govern/kill stuck tasks. Discord on phase/SCORE/KEEP. Strip Cursor coauthor. **On every KEEP: `~/.prometheus/orch_keep_land.sh` pushes main to origin** (catch-up 2026-11-26: 12 commits 7c6aae7→0818e20).
