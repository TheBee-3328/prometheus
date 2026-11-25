# STATUS — PROMETHEUS

**LIVE** 2026-11-25T13:03Z  
**Best (main):** SCORE **20769768** (B5-PROSE KEEP)  
**Prior:** 20852107 (A1) · Δ **−82339**  
**Checkpoint:** tag `CHKP-b5-20769768` → merge commit / `0c3248c`  
**Reproduce:** `nix develop -c ./judge.sh enwik8 .` on `main`  
**Bench cap:** 1 full enwik8 judge  

```
ROUNDTRIP=OK COMPRESSED=20681576 DECOMPRESSOR=88192 SCORE=20769768
COMPRESS_SECONDS=430.91 DECOMPRESS_SECONDS=403.55 PEAK_RSS_MB=3658
```
Adv **10/10**. Peak RSS under Hutter 10GB.

## Closed this session
C1 THROW · B2/B3/B4 REJECT · T0-REFRESH · **B5 KEEP merged**

## Next (T0 rank after prose)
1. Whitespace model (≲5MB cap)  
2. Hibyte/token CM  
3. URL / digit / chrome  

No enwik9 auto. No B2/B3/B4/C1 retries.
