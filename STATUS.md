# STATUS

**Current phase:** 4 (size pass MERGED; handoff notes next)  
**Current best SCORE:** **28080343** (enwik8)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=28051471 DECOMPRESSOR=28872 SCORE=28080343 COMPRESS_SECONDS=35.15 DECOMPRESS_SECONDS=27.11 PEAK_RSS_MB=583`  
**Agent runs used:** 13 / 40  

## Checkpoints

| Checkpoint | Commit | SCORE | Notes |
|---|---|---|---|
| CHKP-tuned | a61e773 / 1a39eee | 28355431 | params search |
| CHKP-size | fa7446b | **28080343** | build-std decomp 28872 B |
| UNFINISHED | agent/p4-orders-mix | — | abandoned / watchdog; do not merge |

## Platform note

Cannot `Task.resume` while subagent still running (Cursor limit). See `prompts/AGENT_WATCHDOG.md`.

## Next three tasks

1. Write HANDOFF.md + SETUP.md (unattended Cursor); slice enwik9 for GitHub.
2. Kill/close P4-ORDERS zombie if still alive; mark checked.
3. Second tuner OR sparse features only after REVIEW if <0.5% gains stack.
