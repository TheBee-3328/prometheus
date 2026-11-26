# P2-MATCH report — THROW (2026-11-26)

**Lane:** `agent/p2-match` · Phase2 match/hash budget  
**Baseline tip:** HASH_BITS=26 BUF_BITS=25 · 25 MiB COMPRESSED **5680631** (reconfirm Δ=0)  
**Gate:** 25 MiB Δ ≤ **−15000** before enwik8 judge  
**Verdict:** **THROW** — gate not cleared after hash (prior) + buf + MIX_WEIGHT + MIN_MIX_LEN + MATCH_CONF  
**RAM:** NO_CAP · all peak RSS recorded · **UNVERIFIED FOR 10 GB**  
**Params:** restored to tip after every family

## Top trials (by Δ bytes vs 5680631)

| tag | family | Δ | wall_s | cpu_s | rss_mb | B/cpu_s | B/MB_rss | note |
|-----|--------|--:|------:|------:|-------:|--------:|---------:|------|
| mix_cool | mix_weight | **−1314** | 100.34 | 99.99 | 3466 | −314.35 | inf | MIX=[100,140,170,200,220] |
| mix_mildcool | mix_weight | −865 | 100.60 | 100.25 | 3466 | −220.66 | inf | MIX=[110,150,180,215,230] |
| conf_soft | match_conf | −216 | 101.76 | 101.41 | 3466 | −78.26 | inf | CONF=[28000…32700] |
| hash_27 | hash | −31 | 102.70 | 102.33 | 4618 | −16.85 | 0.03 | HASH=27 (prior) |

Best Δ **−1314** ≪ gate **−15000** (≈11× short). No enwik8 judge.

## Family summary

| family | result |
|--------|--------|
| HASH (prior) | 24/25 worse; 27 −31 only |
| BUF 23/24/26/27 | 23 +18k; 24 +434; 26/27 **noop** (25 MiB file already covered by BUF=25) |
| MIX_WEIGHT | cool schedules help slightly; hot/early/late/steep all worse; cooler-than-cool overshoots |
| MIN_MIX_LEN | 5 noop; 7 +2847; 8 +123k disaster |
| MATCH_CONF | soft −216; graduated/ramp worse |

## Budget note

Per-CPU-hour is the gate metric; per-GB informational. Best hyps **save** wall/CPU vs tip baseline while saving a few hundred–1.3k bytes — still nowhere near −15 KB. Growing HASH/BUF burns RSS (hash_27 +1.15 GB) for noise-level bits.

## Recommended next orch action

1. **Close P2-MATCH** on this tip — match/hash knobs are saturated; tip mix schedule is near-local-opt.
2. Prefer other Phase2 lanes (CM scale / preprocess / speed) over further match-schedule micro-sweeps.
3. Optional residual: joint `mix_cool` + soft CONF is still expected ≪ −15 KB; do **not** open enwik8 judge without a new hyp class.
