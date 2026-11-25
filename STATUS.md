# STATUS

**Current phase:** LIVE — PRE-V2 word-dict trial → TEMPLATE-MINER → orch  
**Current best SCORE (enwik8):** **21583876**  
**Best line:** `ROUNDTRIP=OK COMPRESSED=21546316 DECOMPRESSOR=37560 SCORE=21583876`  
**bpB:** ≈ **1.724**  
**Gap to 20.5e6:** ≈ 1.05 MB  
**Stop policy:** Operator manual stop.

## Session SCORE path (abbrev)

… → 21778410 → **21583876**

## In flight

1. PRE-V2 word dict (`agent/pre-v2`) — dict built; enable+judge next  
2. PRE-V2 TEMPLATE-MINER after word verdict  

## Parked (NOT merged)

- DIAG-R6 mixer G5: enwik8 SCORE 21395251 but **adversarial 7/10 FAIL** (nondeterminism / cmp). Do not merge until 10/10.
