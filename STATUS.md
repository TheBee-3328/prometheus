# STATUS

**Current phase:** 2→3 (integrate winners, then review)  
**Current best SCORE:** **37298715** (enwik8, order-2 + match)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=36994755 DECOMPRESSOR=303960 SCORE=37298715 COMPRESS_SECONDS=23.74 DECOMPRESS_SECONDS=25.70 PEAK_RSS_MB=161`  
**Agent runs used:** 7 / 40  
**Wall clock budget:** 8 hours from Phase 0 start  

## Phase 2 scoreboard (enwik8, judge-measured)

| Task | SCORE | vs P1 (38254897) | vs MATCH | Verdict |
|---|---|---|---|---|
| P2-MATCH | **37298715** | better | — | MERGED |
| P2-PRE | 37469938 | better | worse | hold → integrate |
| P2-MIXER | 37900419 | better | worse | hold |
| P2-ORDERS | 37944923 | better | worse | hold |
| P2-WORD | 39006952 | **worse** | worse | REJECT |

## Running tasks

| Task ID | Status |
|---|---|
| P2-INTEGRATE | spawning (match + preprocessor) |

## Next three tasks

1. Integrate MATCH + PRE; merge only if SCORE < 37298715.
2. Phase 3 REVIEWER.
3. Phase 3 TUNER on enwik8.
