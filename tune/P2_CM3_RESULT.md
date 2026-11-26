# P2-CM3 report — THROW (2026-11-26)

**Lane:** `agent/p2-cm3`  
**Family:** APM3 richer context (mode 5 + APM2 mix 144)  
**Verdict:** **THROW** vs current tip  

| | SCORE |
|--|--:|
| CM3 enwik8 | **20580794** |
| Tip (P2-CM2 O5=28 MIX=88) | **20566057** |
| Δ vs tip | **+14737** (worse) |

Agent briefly mislabeled KEEP against stale tip **20686092** (pre-CM2). Orch rejects merge.

**slice25:** m5_a144 Δ=−26892 vs old baseline 5643811 (gate aesthetic only; enwik8 loses to CM2 tip).

**Resources (judge):** wall 777.42s · cpu 774.38s · peak RSS 3658 MB · RAM·time 0.7714 GB·h · UNVERIFIED FOR 10 GB

Do not merge. Params restored to tip-equivalent on this branch.
