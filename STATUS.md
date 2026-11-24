# STATUS

**Current phase:** LIVE — PRE-V2 word-dict trial → TEMPLATE-MINER → orch  
**Current best SCORE (enwik8):** **21583876**  
**Best line:** `ROUNDTRIP=OK COMPRESSED=21546316 DECOMPRESSOR=37560 SCORE=21583876`  
**bpB:** ≈ **1.724**  
**Gap to 20.5e6:** ≈ 1.05 MB  
**Stop policy:** Operator manual stop. Unpaused for PRE-V2 dual trials.

## Session SCORE path (abbrev)

28080343 → … → 21778410 → **21583876**

## In flight

1. **PRE-V2 word dict** (`prompts/PRE_V2_WORD_DICT_CONTINUE.md`) — first  
2. **PRE-V2 TEMPLATE-MINER** (`prompts/PRE_V2_TEMPLATE_MINER_CONTINUE.md`) — after word verdict  
3. Continue DIAG/tuner after both KEEP/THROW

## Rules

- Separate keep/throw; THROW on one does not cancel the other  
- enwik8 only for gate; one heavy compress at a time; `nix develop -c …`  
- Merge only ROUNDTRIP=OK and SCORE ≤ 21583876 (or newer main best)
