# P2-O6 — KEEP (2026-11-26)

**Lane:** `agent/p2-o6` @ tip `b9e8668` (SCORE 20566057)  
**Hypothesis family:** ORDER6 bit grow + ORDER6_MIX_WEIGHT (+ mild O3/O4)  
**Tip slice25:** **5625924** (WALL≈101.9s, RSS≈4120 MB)  
**Winner:** `ORDER6_BITS=30` + `ORDER6_MIX_WEIGHT=72` + `ORDER3_BITS=28` + `ORDER4_BITS=28`  
(assert HashOrderModel bits `12..=30`; was O6=26/mix=32, O3=27, O4=27)

## Trials (slice25 vs tip 5625924) — selected

| ID | Change | Bytes | Δ | WALL_s | RSS_KB | Gate |
|--|--|--:|--:|--:|--:|--|
| tip | baseline | 5625924 | 0 | 101.90 | 4120320 | — |
| o6b28 | BITS=28 | 5623360 | −2564 | 104.58 | 4710060 | FAIL |
| o6mix64 | MIX=64 | 5618732 | −7192 | 94.18 | 4120456 | FAIL |
| o6b28_mix72 | BITS=28 MIX=72 | 5612933 | −12991 | 96.35 | 4710204 | FAIL |
| o6b29_mix72 | BITS=29 MIX=72 | 5611611 | −14313 | 106.55 | 5496772 | FAIL |
| o6b29_m72_o34b28 | +O3/O4=28 | 5611045 | −14879 | 92.06 | 6807424 | FAIL |
| o6b30_mix72 | BITS=30 MIX=72 | 5610840 | −15084 | 103.30 | 7069588 | **PASS** |
| **o6b30_m72_o34b28** | **BITS=30 MIX=72 O3=O4=28** | **5610357** | **−15567** | 105.28 | 8380312 | **PASS** |

Full log: `tune/p2_o6_trials.csv` (38 trials). Mix sweet spot ≈64–76; bit grow alone weak until MIX↑ unlocks larger table (same pattern as P2-CM2 on ORDER5).

## enwik8 judge

```
ROUNDTRIP=OK COMPRESSED=20327675 DECOMPRESSOR=87488 SCORE=20415163
COMPRESS_SECONDS=415.26 DECOMPRESS_SECONDS=412.57 PEAK_RSS_MB=8391
CPU_TOTAL_SECONDS=824.84 WALL_TOTAL_SECONDS=827.83 RAM_TIME_GB_H=1.8843
TIME_LIMIT=PASS RAM_LIMIT=PASS
ARCHIVE_SHA256=894e04ce18df2820d45fc56c083f5c031bc94c47616e37a8020c801e2db33a0f
```

Δ SCORE vs tip 20566057: **−150894**

## Adversarial

`nix develop -c ./tests/adversarial_roundtrip.sh` → **PASS=10 FAIL=0** (exclusive after judge).

## Verdict

**KEEP** — SCORE < tip, ROUNDTRIP=OK, TIME/RAM PASS, adv 10/10.  
Do **not** merge without orch.

## Notes

- ORDER6 bit grow alone ≪ gate; MIX↑ to ~72 + BITS=30 clears gate; mild O3/O4=28 adds ~0.5 KB.
- Assert raised `12..=28` → `12..=30` (temporary ceiling for o6 grow).
- No MATCH2 / CM3 APM mode5 / neural.

## RAM

NO_CAP · peak RSS 8391 MB enwik8 · **UNVERIFIED FOR 10 GB**

**Adv:** **10/10 PASS** (exclusive adversarial_roundtrip.sh)
