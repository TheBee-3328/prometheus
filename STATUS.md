# STATUS

**Current phase:** 2 (parallel builders; merging winners)  
**Current best SCORE:** **37298715** (enwik8, order-2 + match)  
**Best measured on:** enwik8  
**Best line:** `ROUNDTRIP=OK COMPRESSED=36994755 DECOMPRESSOR=303960 SCORE=37298715 COMPRESS_SECONDS=23.74 DECOMPRESS_SECONDS=25.70 PEAK_RSS_MB=161`  
**Agent runs used:** 7 / 40 (P0, P1, P2×5 spawned; MATCH+PRE measured)  
**Wall clock budget:** 8 hours from Phase 0 start  

## Running / pending

| Task ID | Status | SCORE |
|---|---|---|
| P2-MATCH | MERGED | 37298715 |
| P2-PRE | PASS pending rebase | 37469938 |
| P2-ORDERS | running | — |
| P2-WORD | running | — |
| P2-MIXER | running | — |

## Known risks

- Nix required. Parallel branches conflict on model.rs — rebase losers onto new main.
- PRE improves vs Phase1 but loses to MATCH; rebase+combine later.
- Do not claim enwik9 yet.

## Next three tasks

1. Finish ORDERS/WORD/MIXER; merge any that beat 37298715.
2. Rebase PRE onto match (or combine preprocessor + match).
3. Phase 3 reviewer after merge batch.
