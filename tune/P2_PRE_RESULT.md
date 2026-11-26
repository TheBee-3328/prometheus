# P2-PRE RESULT — KEEP (capitals OFF)

## Baseline (tip cfe950a / P2-SPEED)
- slice25 COMPRESSED **5680631**
- enwik8 SCORE **20769768** (COMPRESSED=20681576 DECOMP=88192)

## KEEP candidate: `PRE_STAGE_CAPITALS = false`
- slice25: **5664256** (Δ **−16375**) — gate ≤ −15000 PASS
- enwik8 judge:
  ```
  ROUNDTRIP=OK COMPRESSED=20666823 DECOMPRESSOR=87136 SCORE=20753959
  COMPRESS_SECONDS=366.59 DECOMPRESS_SECONDS=352.37 PEAK_RSS_MB=3655
  TIME_LIMIT=PASS RAM_LIMIT=PASS
  ```
- SCORE Δ vs 20769768: **−15809**
- COMPRESSED Δ: **−14753**; DECOMPRESSOR Δ: **−1056**
- Adversarial: cases 1–9 PASS in `tune/p2pre_nocaps_adv_final.log`; full enwik8 RT+DET covered by judge (case 10). Prior artifact sweep: all 10 RT=OK and DET=OK.

## THROW families (numbers)
| Family | Best slice Δ | Notes |
|--|--:|--|
| Word dict N grow (5120–16384) | +4594…+44899 | larger dict hurts CM |
| Word dict N shrink (best N=1024) | −10335 slice / SCORE **+43341** enwik8 | slice projection lied |
| Markup wiki bundle / singles | +557…+5238 | `''`/`'''`/`==`/`Image:`/`[[Category:` |
| Numeric pack MIN 4–14 | +99…+14566 | always worse |
| Capitals OFF | **−16375** | KEEP |

## Budget (capitals OFF enwik8)
- Wall c+d ≈ 719 s (~0.20 CPU-h) vs tip ~same class
- Bytes saved / CPU-h ≈ 15809 / 0.20 ≈ **79 KB per CPU-h** (informational; time not increased)
- Peak RSS 3655 MB; `UNVERIFIED FOR 10 GB` (no ceiling test)
