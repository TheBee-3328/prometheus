# P2-CM2 — KEEP (2026-11-26)

**Lane:** `agent/p2-cm2` @ tip parent `0818e20` → KEEP `6e59cbc`  
**Hypothesis family:** ORDER5→StateMap and/or ORDER3/ORDER5 bit grow  
**Tip slice25:** **5643811** (WALL≈105s, RSS≈3530 MB)  
**Winner:** `ORDER5_BITS=28` + `ORDER5_MIX_WEIGHT=88` (was 26 / 72)

## Trials (slice25 vs tip 5643811)

| ID | Change | Bytes | Δ | WALL_s | RSS_KB | Gate |
|--|--|--:|--:|--:|--:|--|
| tip | baseline | 5643811 | 0 | 105.02 | 3530516 | — |
| H1 | ORDER5→StateMap (bits=26, mix=72) | 5651203 | +7392 | 104.59 | 3661592 | FAIL |
| H2a | ORDER5_BITS=27 | 5640111 | −3700 | 105.75 | 3727228 | FAIL |
| H2b | ORDER5_BITS=28 | 5638018 | −5793 | 107.73 | 4120376 | FAIL |
| H2c | ORDER3_BITS=28 | 5643401 | −410 | 108.91 | 4185852 | FAIL |
| H2d | O3=28 + O5=28 | 5637546 | −6265 | 106.64 | 4776080 | FAIL |
| H2e | ORDER5_BITS=29 (assert temp↑) | 5636904 | −6907 | 109.98 | 4907004 | FAIL |
| H3 | O5 StateMap + BITS=28 | 5645435 | +1624 | 108.94 | 4644504 | FAIL |
| H4a | O5 BITS=28 + MIX=56 | 5653945 | +10134 | 107.86 | 4120468 | FAIL |
| H4b | **O5 BITS=28 + MIX=88** | **5625924** | **−17887** | 103.81 | 4120588 | **PASS** |

## enwik8 judge

```
ROUNDTRIP=OK COMPRESSED=20478441 DECOMPRESSOR=87616 SCORE=20566057
COMPRESS_SECONDS=394.08 DECOMPRESS_SECONDS=389.53 PEAK_RSS_MB=4232
CPU_TOTAL_SECONDS=780.80 WALL_TOTAL_SECONDS=783.61 RAM_TIME_GB_H=0.8994
TIME_LIMIT=PASS RAM_LIMIT=PASS
ARCHIVE_SHA256=603995dcbffbdeb6f67e4f11cce372d1d64fa29383cc724c18a4c4ee11a8db78
```

Δ SCORE vs tip 20686092: **−120035**

## Adversarial

`nix develop -c ./tests/adversarial_roundtrip.sh` → **PASS=10 FAIL=0** (exclusive after judge).

## Verdict

**KEEP** — SCORE < tip, ROUNDTRIP=OK, TIME/RAM PASS, adv 10/10.  
Do **not** merge without orch.

## Notes / rejects

- ORDER5→StateMap alone and with BITS=28 both **worse** (do not retry).
- ORDER3 bit grow ≈ noop; ORDER5 bit grow alone short of gate; **mix↑ to 88 unlocks** the larger table.
- Assert left at `12..=28`; no StateMap on o5; APM2/SSE/PRE unchanged.

## Next suggestion

- Optional residual: SSE_MODE=3 (warm-gate) + existing APM2 (small).
- Do **not** retry ORDER5→StateMap or SSE-SM.
- Candidate later: ORDER6/WORD/INDIRECT bit grow, or APM2 mix fine-tune.
