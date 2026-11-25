# STATUS

**Current phase:** PRE-V2 word dict KEEP — awaiting orch merge; next TEMPLATE-MINER  
**Current best SCORE (enwik8):** **21365747** (branch `agent/pre-v2`, words ON)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=21279587 DECOMPRESSOR=86160 SCORE=21365747`  
**Prior main best:** 21583876  
**bpB:** ≈ **1.709**  
**Stop policy:** Operator manual stop.

## In flight

1. **PRE-V2 word dict** — **KEEP** (N=4096, cycles 0/15)  
2. **PRE-V2 TEMPLATE-MINER** — next (orch)  
3. Continue DIAG/tuner after both KEEP/THROW

## Rules

- Separate keep/throw; THROW on one does not cancel the other  
- enwik8 only for gate; one heavy compress at a time; `nix develop -c …`  
- Merge only ROUNDTRIP=OK and SCORE ≤ current main best (now beat by this branch)
