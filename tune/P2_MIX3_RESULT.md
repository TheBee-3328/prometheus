# P2-MIX3 — KEEP (2026-11-27)

**Lane:** `agent/p2-mix3` · prior tip SCORE **20353480** (`1d6d1cd` IND2)  
**Winner:** `APM_MODE=3` · `PROSE_SM=1` · `PROSE_BITS=26` · `PROSE_MODE=8` · `STRUCT_MIX=12` · `WORD_BITS=26`  
(IND2 locks unchanged: `WORD_HIT_GATE=1`, `ORDER0=40`, `IND=28/40`, `CROSS=8`)  
**Slice25 tip:** 5594393 · winner 5579352 · Δ=**−15041** (gate PASS ≤ −15000)  
**enwik8:** SCORE=**20268625** · COMPRESSED=20180561 · DECOMP=88064 · ROUNDTRIP=OK · TIME PASS · peak RSS 9290 MB  
(official Hutter ≤10 GB OK; working 9 GB ceiling FAIL — no gate-passing RAM-safer config found)  
**Δ vs tip SCORE:** **−84855**

## Judge
```
ROUNDTRIP=OK COMPRESSED=20180561 DECOMPRESSOR=88064 SCORE=20268625
COMPRESS_SECONDS=429.61 DECOMPRESS_SECONDS=424.26 PEAK_RSS_MB=9290
TIME_LIMIT=PASS RAM_LIMIT=FAIL
```

## Top slice25 trials (Δ vs tip 5594393)

| Δ | tag | knobs |
|---:|-----|-------|
| −15041 | m8_s12_wb | APM_MODE=3 PROSE_SM=1 PROSE_BITS=26 PROSE_MODE=8 STRUCT=12 WORD_BITS=26 |
| −14916 | m8_s16_wb | … MODE=8 STRUCT=16 WORD_BITS=26 |
| −14879 | ram_s8_pb_wb | … MODE=8 STRUCT=8 WORD_BITS=26 |
| −14854 | ram_s12_a232_pb | … MODE=8 STRUCT=12 APM_MIX=232 (no WORD26) |
| −14592 | m8_s10 | … MODE=8 STRUCT=10 |
| −14541 | m8_s12 | … MODE=8 STRUCT=12 |
| −13632 | m3pp_pmode8 | APM_MODE=3 PROSE_SM=1 PROSE_BITS=26 PROSE_MODE=8 |
| −12878 | m3pp_s16_wb | APM_MODE=3 PROSE_SM=1 PROSE_BITS=26 STRUCT=16 WORD_BITS=26 |
| −11611 | m3_psm_pb | APM_MODE=3 PROSE_SM=1 PROSE_BITS=26 |
| −10110 | m3_apm192 | APM_MODE=3 APM_MIX=192 |
| −7605 | apm_mode3 | APM_MODE=3 alone |
| −2689 | apm_192 | APM_MIX=192 alone |
| −2441 | psm1 | PROSE_SM=1 alone |

Solo APM2 / WORD_MIX / ORDER1 / most PROSE_MIX moves were ≪ gate; stacking APM_MIX=192 with mode3+prose hurt vs mode3+prose alone.
RAM-safer drops of WORD_BITS=26 or PROSE_BITS=26 missed the −15KB slice gate.
