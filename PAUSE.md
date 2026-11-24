# PAUSE — operator requested

**When:** 2026-11-24 ~23:12 UTC+1  
**Merged while wrapping:** DIAG-R6 eq8 → main @ SCORE **21583876** (`73ab3c8`)

## Do first on resume

1. Read `STATUS.md`.
2. Re-measure enwik9 on current main (prior run was pre-R5: SCORE 186586969).
3. Continue DIAG toward ≤20.5e6 (gap ≈1.05 MB). Likely: mixer retune after asymptotic eq8.

## Do not

- Spawn agents until operator explicitly resumes.
- Retry rejected R5 hyps (SM o6/o7, alt-salt miss, XOR/ind3 expands, APM×ind1).
- Force-push main.

## Best on main

```
ROUNDTRIP=OK COMPRESSED=21546316 DECOMPRESSOR=37560 SCORE=21583876
```


**RESUMED** 2026-11-25 — PRE-V2 word-dict + TEMPLATE-MINER dual trials (operator).
