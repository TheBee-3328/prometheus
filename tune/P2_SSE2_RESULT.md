# P2-SSE2 — THROW (2026-11-26)

**Lane:** `agent/p2-sse2` @ tip SCORE **20415163** (`cffa2e6` / P2-O6)  
**Family:** SSE residual / SSE_MODE warm-gate (+ SSE2_MIX)  
**Tip slice25:** **5610357** (WALL≈110.4s, RSS≈8380 MB)  
**Gate:** slice Δ ≤ −15000 → **FAIL** (best −9014) · enwik8 not run

## Verdict

**THROW** — bestΔ=**−9014** ≪ gate −15000. Tip params restored (SSE_MODE=0 MIX=224 SSE2=192).

## Best (verified exclusive)

| ID | Change | Bytes | Δ | WALL_s |
|--|--|--:|--:|--:|
| **v_m3w_s2_116** | MODE=3 + SSE2 warm SSE2=**116** | **5601343** | **−9014** | 112.6 |
| v_m3w_s2_118 | MODE=3 + SSE2 warm SSE2=118 | 5601350 | −9007 | 113.8 |
| v_m3_s2_120 | MODE=3 MIX=224 SSE2=120 | 5601353 | −9004 | 121.5 |
| v_m3_s2_112 | MODE=3 SSE2=112 | 5601354 | −9003 | 121.7 |
| v_m3_s2_104 | MODE=3 SSE2=104 | 5601503 | −8854 | 121.9 |
| v_tip_s2_112 | MODE=0 SSE2=112 | 5603325 | −7032 | 98.7 |
| v_m8_s2_112 | MODE=8 warm+nibble SSE2=112 | 5603325 | −7032 | — |

## Mode family @ tip mixes (224/192)

| MODE | Δ | Note |
|--|--:|--|
| 0 tip | 0 | baseline |
| 3 warm-gate | −1785 | alone ≪ gate |
| 4 fine buckets | +1133 | |
| 6 nibble Laplace | +1008 | |
| 5 Apm-SSE | +36343 | |
| 7 last_byte | +48492 | |
| 1/2 StateMap | +3e5 | reject |

## Residual notes

- Warm-gate alone (~−1.8 KB) matches prior CM2 residual hint; threshold T∈{4..64} max ~−1871.
- SSE2_MIX↓ on MODE=3 is the only useful residual: sweet spot **112–120** (~−9.0 KB), then degrades (80→−7.3k, 64→−5.5k).
- Tip MODE=0 SSE2↓ peaks ~−7.0 KB (SSE2=112); SSE_MIX↓ alone regresses under exclusive re-verify (mix200 race false+ earlier).
- Combos (mix200+s2, m228+s2) did not beat MODE=3 + SSE2≈120.
- No O6/MATCH2/CM3 re-sweeps. No merge. RAM NO_CAP · UNVERIFIED FOR 10 GB.

Full log: `tune/p2_sse2_trials.csv` · comps `/tmp/p2-sse2-out/`.
