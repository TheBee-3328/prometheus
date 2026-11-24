# STATUS

**Current phase:** handoff ready — **do not auto-start** until operator says so  
**Current best SCORE:** **28080343** (enwik8)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=28051471 DECOMPRESSOR=28872 SCORE=28080343 …`  
**Agent runs used:** 13 / 40  

## Operator

HANDOFF.md describes multi-hour unattended mode for the **next** session. Live orch loop and TUNER-R2 were started by mistake and **stopped**. Wait for an explicit “start monitoring” / “go” before spawning agents again.

## Docs

- `HANDOFF.md` — orch rules (incl. hours-long run when started)
- `SETUP.md` — environment / auto-approve
- Data parts + `scripts/reconstruct_data.sh`

## Next (when operator starts)

1. Tuner round 2
2. Watchdog + merge loop
3. enwik9 only after enwik8 wins
