# STATUS

**Current phase:** LIVE — DIAG-R6 toward ≤20.5e6  
**Current best SCORE (enwik8, main):** **21778410**  
**Best line:** `ROUNDTRIP=OK COMPRESSED=21740866 DECOMPRESSOR=37544 SCORE=21778410`  
**bpB:** ≈ **1.739**  
**Gap to 20.5e6:** ≈ 1.24 MB  
**Stop policy:** Operator manual stop.  

## Session SCORE path (abbrev)

28080343 → … → 21887167 → **21778410** (main) · R6 pending **21583876** (uncommitted)

## enwik9 (pre-R5 / pre-R6 binary, DECOMP=37512)

```
ROUNDTRIP=OK COMPRESSED=186549457 DECOMPRESSOR=37512 SCORE=186586969
COMPRESS_SECONDS=3427.58 DECOMPRESS_SECONDS=3403.97 PEAK_RSS_MB=4347
```
≈1.492 bpB · beats ~205e6 zstd-style floor on COMPRESSED alone.

## In flight

- DIAG-R6: asymptotic soft-reset eq8 judged SCORE **21583876** (−194534 vs main); awaiting commit + adversarial before merge.

## Next (DIAG-R6)

Mixer retune / further keep validation; re-measure enwik9 after merge.
