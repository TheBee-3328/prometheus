# STATUS

**Current phase:** LIVE — Step 2 (A1 retune in flight; B1 merged)  
**Poll cadence:** **2 minutes**  
**enwik9:** NEVER auto-start

## Best enwik8 (main)

```
ROUNDTRIP=OK COMPRESSED=20824740 DECOMPRESSOR=86880 SCORE=20911620
COMPRESS_SECONDS=400.83 DECOMPRESS_SECONDS=400.88 PEAK_RSS_MB=3562
```
**bpB:** ≈ **1.666** · **Gap to 20.5e6:** ≈ 0.41 MB

## enwik9 manual, may be stale

SCORE **186586969** (old binary)

## READY FOR MANUAL ENWIK9 CHECK

- main @ B1-HASH + word dict N=4096
- Larger word dicts 16k/48k
- Possibly larger hash tables for enwik9 scale

## In flight

- **A1-RETUNE** on dict+B1 baseline (rebase may be needed)
- A4 numeric, B2 mixer, B3 contexts, C1 template analysis — queued

## Done

- A3 word dict KEEP · B1 hash grow KEEP
