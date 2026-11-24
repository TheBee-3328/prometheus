# STATUS

**Current phase:** handoff ready (unattended setup)  
**Current best SCORE:** **28080343** (enwik8)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=28051471 DECOMPRESSOR=28872 SCORE=28080343 COMPRESS_SECONDS=35.15 DECOMPRESS_SECONDS=27.11 PEAK_RSS_MB=583`  
**Agent runs used:** 13 / 40  

## Checkpoints

| ID | SCORE | Commit |
|---|---|---|
| CHKP-size (best) | 28080343 | fa7446b |
| CHKP-tuned | 28355431 | a61e773 |
| UNFINISHED | P4-ORDERS-MIX | abandoned — do not merge |

## Docs for next session

- `HANDOFF.md` — orchestrator
- `SETUP.md` — AI setup agent (Nix, data parts, auto-approve)
- `prompts/ORCHESTRATOR_UNSTICK.md`, `prompts/AGENT_WATCHDOG.md`
- `data/README.md` + `scripts/reconstruct_data.sh`

## Next three tasks

1. Setup agent: reconstruct data + smoke judge on new machine/session.
2. Tuner round 2 / REVIEWER per <0.5% rule before big features.
3. enwik9 judge only after another enwik8 win.
