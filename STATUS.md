# STATUS

**Current phase:** 3 (fix P0 from review, then tuner)  
**Current best SCORE:** **36483469** (enwik8) — claim lossless only for enwik-like text until P0 fixed  
**Best line:** `ROUNDTRIP=OK COMPRESSED=36179509 DECOMPRESSOR=303960 SCORE=36483469 COMPRESS_SECONDS=30.79 DECOMPRESS_SECONDS=22.12 PEAK_RSS_MB=344`  
**Agent runs used:** 9 / 40  

## Running

| Task | Status |
|---|---|
| P3-FIX-PRE | spawning (escape 0x01–0x08 markers) |

## Next

1. Fix preprocessor escaping; re-run adversarial + judge; SCORE must stay ≤ 36483469.
2. TUNER.
3. Optional: size pass (build-std / eh_frame).
