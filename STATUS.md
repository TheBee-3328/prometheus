# STATUS

**Current phase:** LIVE — Step 1 baseline locked; starting Step 2 Group A/B  
**Poll cadence:** **2 minutes**  
**enwik9:** NEVER auto-start (manual only)

## Locked enwik8 baseline (best.json)

```
ROUNDTRIP=OK COMPRESSED=21279587 DECOMPRESSOR=86160 SCORE=21365747
COMPRESS_SECONDS=307.98 DECOMPRESS_SECONDS=290.25 PEAK_RSS_MB=1834
```
**bpB:** ≈ **1.702** (words-on N=4096, adv 10/10)  
**On main?** No yet — `main` tip still SCORE **21583876** until words KEEP merge.  
**Gap to 20.5e6:** ≈ 0.87 MB (vs locked) / ≈ 1.05 MB (vs main tip)

## enwik9 manual, may be stale

```
ROUNDTRIP=OK COMPRESSED=186549457 DECOMPRESSOR=37512 SCORE=186586969
COMPRESS_SECONDS=3427.58 DECOMPRESS_SECONDS=3403.97 PEAK_RSS_MB=4347
```

## READY FOR MANUAL ENWIK9 CHECK

- main commit: (update after words merge)
- Flagged: word_dict 16k/48k candidates; R6 word-SM SCORE 21263097 (needs adv); table sizes TBD

## Agents

**Stopped** at Step-1 lock (process groups killed). Orphans cleared.

## Next (Step 2)

- A1 retune on dict-coded input → A2 caps (exists) → A3 word dict (KEEP path) → A4 numeric  
- B1–B3 in parallel worktrees  
- C1 template analysis if abcd>3%  
- No enwik9 runs
