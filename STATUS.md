# STATUS

**Current phase:** LIVE unattended — TUNER-R2 in flight  
**Current best SCORE:** **28080343** (enwik8)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=28051471 DECOMPRESSOR=28872 SCORE=28080343 …`  
**Agent runs used:** 14 / 40 (TUNER-R2 spawned)  
**Wall clock:** session started 2026-11-24 ~03:04 UTC+1  

## Operator

Unattended orch loop **RUNNING**. Monitor freely. Soft failures logged and continued; stop only on critical risk.

## In flight

| Task | Branch | Soft/Hard | Goal |
|---|---|---|---|
| TUNER-R2 | `agent/tuner-r2` | 50 / 70 min | Beat SCORE 28080343 via params search seeded from current best |

## Next queue

1. Merge TUNER-R2 if PASS else log FAIL
2. If <0.5% gain: REVIEWER+TUNER again before big features; else sparse/XML ensemble with match
3. enwik9 only after clear enwik8 win

## Docs

- `HANDOFF.md` — orch rules
- `SETUP.md` — environment / auto-approve
- `prompts/AGENT_WATCHDOG.md` — timeouts
