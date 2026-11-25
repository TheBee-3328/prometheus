# B5-PROSE — KEEP

Branch: `agent/b5-prose`  
Baseline: `SCORE=20852107` (COMPRESSED=20765227 DECOMPRESSOR=86880)

## Full enwik8 judge

```
ROUNDTRIP=OK COMPRESSED=20681576 DECOMPRESSOR=88192 SCORE=20769768
COMPRESS_SECONDS=430.91 DECOMPRESS_SECONDS=403.55 PEAK_RSS_MB=3658
```

| Metric | Baseline | After | Δ |
|--------|--------:|------:|--:|
| COMPRESSED | 20765227 | 20681576 | **−83651** |
| DECOMPRESSOR | 86880 | 88192 | +1312 |
| SCORE | 20852107 | **20769768** | **−82339** |

## Slice (25 MiB head)

Baseline COMPRESSED=5699220. Best Δ=**−18589** (gate ≤ −15000).

Winner: `PROSE_MODE=2` (prev+cur word hash), `PROSE_MIX=24`, `PROSE_BITS=25`, hit-gated mix; `WORD_HIT_GATE=1`.

## What helped

- **Word hit-gate** (skip mix on checksum miss / cold slot) — alone ≈ −9k slice
- **Prose prev+cur ensemble** at low–mid weight with large table + hit-gate — stacks to ≈ −18.6k slice
- Larger prose table (24→25 bits) and mix≈24 (peak; 28+ regresses)

## What hurt

- High prose mix (48+) without hit-gate
- Folding prev into WordModel hash (WORD_CTX_MODE 1/2) — sparse/cold, +88k slice
- Skip-gram / position-only / StateMap prose at tried weights
- Match-gated prose alone

See `tune/b5_trials.csv`.
