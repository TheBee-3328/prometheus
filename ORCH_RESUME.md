# ORCH RESUME — improvement loop (2026-11-26T14:54Z)

## Live lanes
- `prometheus-p2-cm2` / `agent/p2-cm2` — GATE PASS mix88 → enwik8 judge in flight
- `prometheus-p2-cm3` / `agent/p2-cm3` — ACTIVE (MATCH2 THROW)
- 3m monitor `AGENT_LOOP_TICK_prometheus_orch`
- Discord: `~/.prometheus/discord_notify.js`

## Tip
enwik8 Best **20686092** · main `0818e20` (must stay = origin/main after every KEEP)

## On KEEP (mandatory — do not skip push)
1. Merge branch → main (strip Cursor coauthor)
2. Update Best / STATUS / best.json / LOG
3. **`~/.prometheus/orch_keep_land.sh <SCORE> [agent-branch] [note]`** — pushes `main` (+ branch) to origin
   - or at minimum: `~/.prometheus/orch_push_main.sh "KEEP …"`
4. Discord KEEP with Resources
5. Spawn next lane

**Root cause of 24h+ no-push:** merges landed on local main only; origin stayed at B6-WS log. Never again — push is part of KEEP, not optional.

## Do NOT
- Idle forever when queue empty — invent next CM/mixer/indirect family
- Restart duplicate adv waiters / mix sweeps already gated
- Neural until §2.4 determinism PASS
- Leave main ahead of origin overnight
