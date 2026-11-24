# STATUS

**Current phase:** LIVE unattended — next: P5-SPARSE-XML  
**Current best SCORE:** **28042709** (enwik8)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=28013837 DECOMPRESSOR=28872 SCORE=28042709 …`  
**Agent runs used:** 15 / 40 (TUNER-R2 merged)  
**Prior:** 28080343 → 28042709 (−37634, −0.13%)  

## Operator

Unattended orch loop **RUNNING**. Monitor freely.

## Last merge

| Task | Commit | SCORE | Notes |
|---|---|---|---|
| TUNER-R2 | `b8465d4` / merge on main | **28042709** | ORDER2_RESCALE=100, MIX_WEIGHT=[120,160,190,230,245]; adv 10/10 |

## In flight / next

1. **P5-SPARSE-XML** — sparse/XML contexts that *ensemble with* match (not replace; P4-ORDERS lesson)
2. If <0.5% again → REVIEWER + another TUNER before more features
3. enwik9 only after clear enwik8 win

## Caps

Budget 40 agent runs / 8 wall hours. Do not retry P2-WORD or P4-ORDERS-MIX briefs.
