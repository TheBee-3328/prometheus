# P2-CM4 — KEEP (2026-11-27)

**Lane:** `agent/p2-cm4` @ tip `29f6171` (SCORE 20268625, MIX3)  
**Hypothesis family:** ORDER3/ORDER4 mix rebalance + ORDER7 bit grow  
**Tip slice25:** **5579352** (WALL≈118.9s, RSS≈9300 MB)  
**Winner:** `ORDER3_MIX_WEIGHT=100` + `ORDER4_MIX_WEIGHT=88` + `ORDER7_BITS=27` + `ORDER7_MIX_WEIGHT=32`  
(was O3_MIX=160, O4_MIX=48, O7=25/16)

## Trials (slice25 vs tip 5579352) — selected

| ID | Change | Bytes | Δ | WALL_s | RSS_KB | Gate |
|--|--|--:|--:|--:|--:|--|
| tip | baseline | 5579352 | 0 | 118.87 | 9299748 | — |
| o4mix80 | O4_MIX=80 | 5571562 | −7790 | 102.97 | 9299836 | FAIL |
| o3mix128 | O3_MIX=128 | 5571335 | −8017 | 122.27 | 9299796 | FAIL |
| o3o4_m144_80 | O3=144 O4=80 | 5566672 | −12680 | 110.66 | 9299680 | FAIL |
| o3o4_m128_96 | O3=128 O4=96 | 5563746 | −15606 | 109.73 | 9300028 | **PASS** |
| o3o4_m120_96 | O3=120 O4=96 | 5562624 | −16728 | 109.52 | 9299976 | **PASS** |
| o3o4_m128_96_o7b27 | +O7=27/32 | 5560798 | −18554 | 109.82 | 9594824 | **PASS** |
| o3o4_m112_96_o7b27 | O3=112 O4=96 O7 | 5558642 | −20710 | 111.00 | 9594656 | **PASS** |
| **o3o4_m100_88_o7b27** | **O3=100 O4=88 O7=27/32** | **5557124** | **−22228** | 108.92 | 9595024 | **PASS** |
| o3o4_m96_88_o7b27 | O3=96 O4=88 O7 | 5557128 | −22224 | 109.94 | 9594832 | **PASS** |

Full log: `tune/p2_cm4_trials.csv`. Lowering O3 mix (160→~100) + raising O4 (48→88) is additive; O7 BITS 25→27 (+MIX 16→32) stacks ~−3 KB more at +~295 MB RSS.

## enwik8 judge

```
ROUNDTRIP=OK COMPRESSED=20075858 DECOMPRESSOR=88064 SCORE=20163922
COMPRESS_SECONDS=427.15 DECOMPRESS_SECONDS=419.33 PEAK_RSS_MB=9577
CPU_TOTAL_SECONDS=843.35 WALL_TOTAL_SECONDS=846.48 RAM_TIME_GB_H=2.1990
TIME_LIMIT=PASS RAM_LIMIT=FAIL
ARCHIVE_SHA256=183ed9e3798894a4bf73cdb14a44ea061cdbcb03a9d422f17bc743c2296b4993
```

Δ SCORE vs tip 20268625: **−104703**

Working 9 GB ceiling FAIL (peak 9577 MB); official 10 GB headroom remains (~423 MB). Same class as MIX3 tip (9290 MB working-ceiling FAIL).

## Verdict

**KEEP** — SCORE < tip, ROUNDTRIP=OK, TIME PASS.  
Do **not** merge without orch.

## Notes / rejects

- O5 mix↑ alone ≈ −2.5 KB; stacking O5 onto O3/O4 regresses vs O3/O4 alone.
- O6 mix fine-tune near 72 ≈ noise / slight regress.
- WORD_MIX↑ toxic; APM2_MIX noop under APM_MODE=3.
- O7_BITS=28 clears more bytes but slice RSS ~9988 MB — too tight for 10 GB enwik8.
- MATCH/HASH not re-swept (MATCH2 saturated ≪ gate).
- Locked MIX3/IND2 knobs untouched.

## RAM

NO_CAP · peak RSS 9577 MB enwik8 · under 10 GB official · **working 9 GB FAIL**
