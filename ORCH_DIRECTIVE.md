# ORCH — P2-CM2 KEEP DONE 2026-11-26T15:44Z

**Verdict: KEEP** on `agent/p2-cm2` @ `6e59cbc` (+ follow-up result/adv notes).

- Winner: ORDER5_BITS=28 + ORDER5_MIX_WEIGHT=88
- slice25 Δ=−17887 (gate PASS); enwik8 SCORE=**20566057** (Δ −120035 vs 20686092)
- ROUNDTRIP=OK TIME/RAM PASS peak RSS 4232 MB; adv **10/10**
- REJECT: ORDER5→StateMap (worse); ORDER3 grow alone ≈noop
- Do **not** merge without orch. Strip Cursor coauthor. No neural. No SSE-SM retry.
- Next: optional SSE_MODE=3 warm-gate residual; else ORDER6/WORD bit grow family.
- Details: `tune/P2_CM2_RESULT.md`
