# STATUS

**Current phase:** 4 (size pass after orders-mix FAIL)  
**Current best SCORE:** **28355431** (enwik8)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=28051471 DECOMPRESSOR=303960 SCORE=28355431 … PEAK_RSS_MB=584`  
**Agent runs used:** 12 / 40  

## Recent

- P4-ORDERS-MIX abandoned after 2 attempts worse than best (30.9M, 28.8M).
- Next: shrink decompressor (every byte counts) and/or tune round 2.

## Next three tasks

1. Size-optimize decompressor (eh_frame / panic_abort / LTO already on — push further).
2. Second tuner round on params.
3. enwik9 only after another clear enwik8 win.
