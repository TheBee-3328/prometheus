# STATUS

**Current phase:** LIVE — Step 2 after word-dict KEEP merge  
**Poll cadence:** **2 minutes**  
**enwik9:** NEVER auto-start (manual only)

## Locked / current best enwik8 (on main after merge)

```
ROUNDTRIP=OK COMPRESSED=21279587 DECOMPRESSOR=86160 SCORE=21365747
COMPRESS_SECONDS=307.98 DECOMPRESS_SECONDS=290.25 PEAK_RSS_MB=1834
```
**bpB:** ≈ **1.702** · **Gap to 20.5e6:** ≈ 0.87 MB

## enwik9 manual, may be stale

```
ROUNDTRIP=OK COMPRESSED=186549457 DECOMPRESSOR=37512 SCORE=186586969
```

## READY FOR MANUAL ENWIK9 CHECK

- **main tip:** (after this merge) word dict N=4096 KEEP
- Flagged larger dicts: `generated/word_dict_16384.bin` (201320 B), `word_dict_49152.bin` (647120 B) — within ~0.2% check not yet done; candidates for manual enwik9
- Parked: DIAG-R6 word-SM SCORE 21263097 (needs fresh adv); mixer G5 (adv flake history)

## Next (Step 2)

- **A1** retune on dict-coded input (20–30MB slice → top3 enwik8)
- **A2** caps already present — verify/skip if SCORE-neutral
- **A3** word dict — **DONE** (KEEP N=4096)
- **A4** numeric fields
- **B1–B3** parallel model worktrees
- **C1** TEMPLATE-MINER analysis (3% gate)
- No enwik9 auto-runs
