# STATUS

**Current phase:** LIVE unattended — P5-SPARSE-XML in flight  
**Current best SCORE:** **28042709** (enwik8)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=28013837 DECOMPRESSOR=28872 SCORE=28042709 …`  
**Agent runs used:** 15 (informational; **no auto-stop**)  
**Stop policy:** Operator manual stop (~15 h from 2026-11-24 ~03:33 UTC+1). Old 40-run / 8-hour caps **disabled**.  

## Operator

Unattended orch loop **RUNNING** until you stop it. Monitor freely.

## Last merge

| Task | Commit | SCORE | Notes |
|---|---|---|---|
| TUNER-R2 | `b8465d4` / merge on main | **28042709** | ORDER2_RESCALE=100, MIX_WEIGHT=[120,160,190,230,245]; adv 10/10 |

## In flight / next

1. **P5-SPARSE-XML** — sparse/XML contexts that *ensemble with* match
2. If FAIL / <0.5%: REVIEWER and/or TUNER-R3 (finer MIX grid from TUNER-R2 notes)
3. enwik9 only after clear enwik8 win

## Caps

Do not retry P2-WORD or P4-ORDERS-MIX briefs. Watchdog soft/hard timeouts still apply per task.
