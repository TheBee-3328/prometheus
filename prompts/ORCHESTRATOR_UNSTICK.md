# Orchestrator unstick / wake prompt

Use this when the **orchestrator** (or a long-running orch session) looks frozen: no merges, no STATUS/LOG updates, no new spawns, while subagents may still be thrashing.

Copy, paste, and send as the next user message (or `Task`/`resume` payload). Do **not** invent a new phase before this summary.

---

## Template (orchestrator)

```
You are the ORCHESTRATOR. You may be stalled. Re-synchronize and continue — do not restart the project from Phase 0.

Read STATUS.md, LOG.md, and the last 20 commits. Summarize in 10 lines:
- current best SCORE (judge-measured only) and which commit/branch holds it
- what has failed twice (do not retry the same brief a third time)
- where time was wasted (parallel conflicts, regressions, zombie agents)
- any in-flight agent branches and whether they beat current best
- Nix rule: all builds via `nix develop -c …`
- disk/RAM caution if host is tight

Then choose the **single** next task with the best expected gain per cost, write its brief using TASK_TEMPLATE.md, spawn or resume that agent only, and continue the orchestration loop from the **current** phase in STATUS.md.

Rules:
- Merge only if `./judge.sh enwik8 .` improves SCORE (or equal with smaller code) and ROUNDTRIP=OK.
- If two consecutive phases produced under 0.5% SCORE improvement, run a REVIEWER and a TUNER before adding any new feature.
- Cap budget (STATUS.md agent runs / wall clock). Check before each spawn.
- Never claim enwik9 results without `nix develop -c ./judge.sh enwik9 .`.
- After the next merge or hard FAIL, update LOG.md and STATUS.md and push main.

After the summary: execute the chosen next step immediately (spawn/resume/merge/kill-zombie). Stay live; do not wait for a human to resume you.
```

---

## Variant: steer a **zombie subagent** (edit before send)

Fill the bracketed fields for the actual stuck task, then `resume` that agent id.

```
STOP conflicting work. You are agent [TASK ID] on branch [branch]. The orchestrator has [ABANDONED|SUPERSEDED] this task.

Current best SCORE on main: [SCORE] at commit [hash].
Your last measured numbers (if any): [SCORE line or "none"].
Verdict: [you are behind best / duplicate work / wrong approach].

Required actions NOW:
1. Do not edit more model code unless you already have a judge line with SCORE < [BEST].
2. If you have ROUNDTRIP=OK and SCORE < [BEST], commit, push [branch], and report the exact judge line + commit hash.
3. Otherwise: stop builds/judges, leave the tree, and report FAIL with what you tried and numbers — then exit.
4. Do not touch judge files, LOG.md, STATUS.md, or other agents' branches.

One paragraph final report only.
```

### Filled example (P4-ORDERS-MIX abandon)

```
STOP conflicting work. You are agent P4-ORDERS-MIX on branch agent/p4-orders-mix. The orchestrator has ABANDONED this task after two FAILs.

Current best SCORE on main: 28355431 at commit a61e773 / edb136b.
Your last measured numbers: SCORE 30912214 and 28809463 (both worse).
Verdict: you are behind best; approach failed twice — do not retry.

Required actions NOW:
1. Do not edit more model code.
2. Stop builds/judges, leave the tree, and report FAIL with what you tried and numbers — then exit.
3. Do not touch judge files, LOG.md, STATUS.md, or agent/p4-size.

One paragraph final report only.
```

---

## Variant: nudge a **live** agent that is mid-validate (gentle)

```
You are [TASK ID] on [branch]. Keep going — do not redesign.

Current best to beat: [SCORE].
You already have [evidence: e.g. DECOMPRESSOR=28872, COMPRESSED unchanged].

Finish: ensure `nix develop -c ./judge.sh enwik8 .` and adversarial pass, then commit + push [branch] with the exact SCORE line in the commit message body. Report PASS/FAIL + hash. Budget remaining: [N] minutes.
```
