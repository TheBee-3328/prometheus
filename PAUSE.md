# PAUSE — operator requested

**When:** 2026-11-24 ~23:12 UTC+1 (operator: pause after current subagents finish)  
**Why:** Manual halt; do not spawn further agents after in-flight work ends.

## In flight at pause request

- **DIAG-R6** (`/tmp/workspace/prometheus-diag-r6`, `agent/diag-r6`): asymptotic soft-reset **eq8 (mode7)**  
  - Judge: `ROUNDTRIP=OK COMPRESSED=21546316 DECOMPRESSOR=37560 SCORE=21583876` (−194534 vs main 21778410)  
  - Adversarial: **10/10 PASS**  
  - Still uncommitted/unpushed at pause request; orch will merge if agent lands the keep before exit.

## Do first on resume

1. Read `STATUS.md` + this file.
2. If `agent/diag-r6` has SCORE &lt; 21778410 + ROUNDTRIP/adversarial PASS and is not on main: merge → push main → update LOG/STATUS.
3. Re-measure enwik9 on post-merge binary (prior enwik9 was pre-R5: SCORE 186586969).
4. Continue DIAG toward ≤20.5e6 (gap was ≈1.24 MB pre-R6; ≈1.05 MB if eq8 merges).

## Do not

- Spawn new agents until operator explicitly resumes.
- Retry rejected R5 hyps (SM o6/o7, alt-salt miss, XOR/ind3 expands, APM×ind1).
- Force-push main.

## Best on main (until R6 merge)

```
ROUNDTRIP=OK COMPRESSED=21740866 DECOMPRESSOR=37544 SCORE=21778410
```

## enwik9 (pre-R5 binary)

```
ROUNDTRIP=OK COMPRESSED=186549457 DECOMPRESSOR=37512 SCORE=186586969
```
