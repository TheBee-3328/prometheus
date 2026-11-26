# STATUS — PROMETHEUS Phase 2

**LIVE** 2026-11-26T05:36Z · orch **KEEP** · main `333de83` · RAM ~13 Gi avail  
**Host CPU:** AMD Ryzen 5 5600X (6C/12T)  
**Geekbench5 T:** **1650 assumed** (not measured yet — GB5 run queued) → H≈42.42 h/dir · working 0.8×H≈33.94 h/dir (122182 s)  
**Best:** enwik8 **20686092** · enwik9 **173961591** — both **UNVERIFIED FOR 10 GB** (no ceiling test yet; prior 9 GB working gate retired)  
**Limits:** `PROMETHEUS_PHASE2_LIMITS.md` — time is the budget; default RAM=NO_CAP  
**License:** GPLv3-or-later

## Active
| Lane | Branch | State |
|--|--|--|
| P2-SPEED | `agent/p2-speed` | **KEEP** on main (−24% wall, bit-id) |
| P2-MATCH | `agent/p2-match` | **THROW** `796353a` |
| P2-PRE | `agent/p2-pre` | **KEEP merged** (nocaps; tip superseded) |
| P2-CM | `agent/p2-cm` | **KEEP merged** SCORE=**20686092** (APM2 mix=160 + nocaps) |

## Queue
Neural blocked

## Orch
Continuous 3m monitor. Govern/kill stuck tasks. Discord on updates. Strip Cursor coauthor trailer on commits.
