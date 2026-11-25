# B4-HMIX — REJECT

**Branch:** `agent/b4-hmix`  
**Baseline tip:** `0c583bb` / A1 weights (ORDER0=64 CROSS=16 IND=32 ORDER5=72)  
**Locked SCORE:** 20852107 (COMPRESSED=20765227 DECOMP=86880)  
**Slice:** `tune/b4_25mb.bin` (first 25 MiB enwik8)  
**Slice baseline:** **5699220**

## Verdict

**REJECT** — best slice Δ **−1280** (H2c). Abort gate requires Δ ≤ **−15000**. No full enwik8 judge. Extra APM stage not tried (hierarchical path never cleared the −15KB bar).

## Trials (`tune/b4_trials.csv`)

| Hyp | Idea | Slice | Δ |
|-----|------|------:|--:|
| baseline | A1 sequential mix | 5699220 | 0 |
| H1a | Two-layer short vs long, fixed GROUP=128 | 5773465 | +74245 |
| H2a | Match-active weight set (O0/O5/IND/CROSS/WORD) | 5713053 | +13833 |
| H2b | Mild match-on IND=40 CROSS=22 | 5703939 | +4719 |
| **H2c** | **Match-on mute IND=24 CROSS=10** | **5697940** | **−1280** |
| H2d | Stronger mute IND=16 CROSS=4 | 5700865 | +1645 |
| H2e | IND=24 CROSS=8 | 5698176 | −1044 |
| H3a | Dual-rate adaptive bias on match MIX_WEIGHT | 5760011 | +60791 |
| H4 | Short vs long + adaptive `group_w[mb×bp]` | 5896068 | +196848 |

## Notes

- Regrouping short/long **without** full weight recalibration destroys A1 sequential calibration (H1/H4).
- Context-selected match-active sets help only as a tiny mute of cross/ind under match (~1.3KB); not lane-scale.
- Dual-rate / loss-EMA match-mix structure regresses hard at the rates tried (H3a).
- Do **not** repeat B2 1D SSE2 grids, B3 match-cand/APM2/prev-word/HASH_ORDER7, or C1 template THROW.

## Code state

Mixer/`params.rs` restored to A1 baseline after measurements. Artifacts kept: `tune/b4_trials.csv`, `tune/b4_slice_eval.sh`, `tune/b4_25mb.bin`.
