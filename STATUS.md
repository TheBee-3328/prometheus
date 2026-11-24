# STATUS

**Current phase:** 3 complete → next: combine held models / size pass / more tune  
**Current best SCORE:** **28355431** (enwik8)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=28051471 DECOMPRESSOR=303960 SCORE=28355431 COMPRESS_SECONDS=37.04 DECOMPRESS_SECONDS=28.11 PEAK_RSS_MB=584`  
**Ratio:** compressed ≈ 3.57× vs raw; SCORE/1e8 ≈ 0.284  
**Agent runs used:** 11 / 40  

## Scoreboard

| Build | SCORE |
|---|---|
| Store | 100299876 |
| Order-2 | 38254897 |
| Match+pre | 36483469 |
| **Tuned match+pre** | **28355431** |

## Next three tasks

1. Second tune round OR integrate held P2-ORDERS/MIXER onto tuned base.
2. Decompressor size pass (build-std / eh_frame).
3. enwik9 milestone only after another enwik8 improvement batch.
