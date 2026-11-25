# B6-WS — REJECT

Branch: `agent/b6-ws`  
Baseline (25 MiB head, WS off): `COMPRESSED=5680631`  
Gate: slice Δ ≤ **−15000**

## Verdict

**REJECT.** Best slice Δ ≈ **−3133** (`m25_w12_g1_b26`) after 90+ trials; post-mix peek ≈ **−3890** — still ≪ gate. No full enwik8 judge. WS left **off** (main-equivalent); do not merge.

## Best slice rows

| Tag | Δ vs baseline | Notes |
|-----|--------------:|-------|
| `m25_w12_g1_b26` | **−3133** | best gated pre-post |
| `m25_w12_g1_b25` | −3131 | |
| `m25_w14_g1_b25` | −3061 | |
| `m25_w12_g1_b26_post` | −3890 | post-mix; still far from gate |

See `tune/b6_trials.csv`.

## What was tried

- Whitespace CM modes 1–27, mix 8–40, bits 22–26, match-gate on/off
- Post-mixer placement (`WS_POST_MIX`) — small extra gain, not enough

## Why reject

25 MiB gate needs ≈ −15 KB; observed ceiling ≈ −3–4 KB. Not worth full enwik8 SCORE or main merge.

## Restore

Sources restored to main-equivalent (no WS). Experiment artifacts (`ws.rs`, trials CSV) retained on branch for archaeology only.
