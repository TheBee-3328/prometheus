# STATUS

**Current phase:** PAUSED — waiting for DIAG-R6 agent exit; no new spawns  
**Current best SCORE (enwik8):** **21583876**  
**Best line:** `ROUNDTRIP=OK COMPRESSED=21546316 DECOMPRESSOR=37560 SCORE=21583876`  
**bpB:** ≈ **1.724**  
**Gap to 20.5e6:** ≈ 1.05 MB  
**Stop policy:** Operator pause (see PAUSE.md).  

## Session SCORE path (abbrev)

28080343 → … → 21887167 → 21778410 → **21583876**

## Merged

- DIAG-R6 (`73ab3c8`): asymptotic StateMap `n+=1-(n>>3)` (eq≈8)

## enwik9 (pre-R5/R6 binary — remeasure on resume)

```
ROUNDTRIP=OK COMPRESSED=186549457 DECOMPRESSOR=37512 SCORE=186586969
```

## On resume

1. Confirm DIAG-R6 agent exited; no orphan judges.
2. Re-judge enwik9 on current main.
3. Continue toward ≤20.5e6 (mixer retune after eq8, etc.).
