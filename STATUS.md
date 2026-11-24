# STATUS

**Current phase:** LIVE unattended — REVIEWER + TUNER-R3 (two <0.5% wins)  
**Current best SCORE:** **28011126** (enwik8)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=27981374 DECOMPRESSOR=29752 SCORE=28011126 …`  
**Agent runs used:** 16 (informational; no auto-stop)  
**Stop policy:** Operator manual stop (~15 h from ~03:33 UTC+1).  

## Operator

Orch **RUNNING** until you stop it.

## Merges this session

| Task | SCORE | Δ |
|---|---|---|
| TUNER-R2 | 28042709 | −37634 vs 28080343 (−0.13%) |
| P5-SPARSE-XML | **28011126** | −31583 vs 28042709 (−0.11%) |

## In flight / next

1. **P5-REVIEW** — adversarial + determinism + size notes at new baseline  
2. **TUNER-R3** — finer MIX / STRUCT_MIX_WEIGHT / RESCALE around current best  
3. New features only after REVIEWER+TUNER cycle  

## Caps

No P2-WORD / P4-ORDERS-MIX retries. Watchdog timeouts still apply.
