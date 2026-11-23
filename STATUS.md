# STATUS

**Current phase:** 3 (tuner)  
**Current best SCORE:** **36483469** (enwik8, match + preprocessor, escaped markers)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=36179509 DECOMPRESSOR=303960 SCORE=36483469 COMPRESS_SECONDS=32.87 DECOMPRESS_SECONDS=23.31 PEAK_RSS_MB=344`  
**Agent runs used:** 10 / 40  

## Done recently

- P3-FIX-PRE merged: adversarial 10/10 PASS; SCORE equal to pre-fix (no regression).

## Running

| Task | Status |
|---|---|
| P3-TUNER | spawning |

## Next

1. TUNER on params (enwik8 slice → full validate).
2. Optional size pass / combine held Phase2 models.
3. enwik9 only at phase milestone.
