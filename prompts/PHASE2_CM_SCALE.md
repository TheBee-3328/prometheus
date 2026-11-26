# Agent brief — Phase 2 Step 3: bigger context-mixing (pay-for-itself)

**Branch / worktree:** `agent/p2-cm` from B5 tip.  
**Spawn only after** speed lane has landed or orch says go (CPU budget).

## Ablation prior (25 MiB, mix→0 cost = value of component)

| Off | Δ compressed (worse) |
|--|--:|
| SSE | +191299 |
| APM | +93653 |
| ORDER5 | +88668 |
| ORDER3 | +87765 |
| WORD | +86562 |
| ORDER4 | +53229 |
| INDIRECT | +47029 |
| SSE2 | +23945 |
| ORDER6 | +21214 |
| PROSE | +9580 |
| ORDER7 / O0 / O1 / STRUCT | small |

→ Prefer **enriching high-value paths** (SSE/APM chain, o3/o5, word/indirect) over low-value (struct/o1). Whitespace lane already REJECT.

## Goal
Add contexts / mixer depth / SSE-APM chain only if **SCORE↓** under budget rule (bytes per CPU-hour and per GB). Working ceilings 4 h/dir, 9 GB.

## Method
One hypothesis family per cycle. Slice gate −15 KB @ 25 MiB then enwik8. Integer/fixed-point only. Adv 10/10 before KEEP.

## Forbidden
- Neural/LSTM (Phase 2 Step 4 — separate brief later).
- FP; threads; enwik9 auto; ignoring DECOMPRESSOR growth.
