# Agent watchdog & mid-flight steering

## Can the orchestrator message a running subagent?

**No — not with Cursor’s in-IDE `Task` / best-of-n `resume` while the subagent is still running.**

Observed error:

```text
Sub-agent is currently running. You may send the follow-up message when it has completed.
```

This is a **Cursor platform / Task-tool limit**, not a PROMETHEUS repo setting. We cannot “fix” it inside `flake.nix` or `cli-config.json`. Workarounds:

| Approach | When to use |
|---|---|
| Wait for completion notification, then `resume` | Normal path |
| Soft nudge only after WAL idle / process exit | Prefer before killing |
| Kill owned processes + `resume` with STOP/FINISH brief | Soft-stuck (looping judge, no commit) |
| Abandon branch, mark UNFINISHED in LOG, spawn replacement | Hard-stuck past budget |
| Cursor SDK `Agent.send` / follow-ups (outside this IDE Task flow) | Future unattended runner — see SETUP notes |

Orchestrator **must** poll externally (git remote, `judge` processes, agent WAL mtime) and apply timeouts below — do not rely on mid-flight chat.

---

## Timeout policy (orchestrator)

Decide timeout from the **task type**, not a single global number. Clock starts at spawn (or last meaningful progress: new commit, new judge line, WAL activity).

| Task class | Soft timeout (nudge/kill procs) | Hard timeout (mark UNFINISHED) | Progress signals |
|---|---|---|---|
| Judge-only verify | 15 min | 25 min | `ROUNDTRIP=` line, remote commit |
| Size / build flags (P4-SIZE style) | 30 min | 45 min | smaller `prometheus-decompress`, remote commit |
| Single-model builder (enwik8) | 45 min | 60 min | compile, slice numbers, judge line, push |
| Tuner search | 50 min | 70 min | `tune/trials.csv` growth, then full validate |
| Parallel Phase-2 style | 60 min each | 75 min | per-branch push |
| Reviewer (docs + adversarial) | 40 min | 55 min | `REVIEW.md` push |
| enwik9 full judge | 3 h | 5 h | judge line |

**Progress reset:** if WAL mtime, `trials.csv`, or `origin/<branch>` advances, reset the soft clock once. Do not reset forever — hard timeout is absolute from spawn.

**Stuck definition:** soft timeout elapsed AND no progress signal for soft/2 (e.g. 15 min with no WAL/git/judge change).

### On soft timeout

1. Try `resume` with the gentle nudge variant from `ORCHESTRATOR_UNSTICK.md` (only works if agent is **not** running — if still running, skip to step 2).
2. Kill **only** processes clearly owned by that task (paths under its worktree / its `judge.sh`). Never kill another agent’s compress.
3. Wait up to 5 min for commit/push or idle.
4. If still no PASS commit: hard-fail path.

### On hard timeout

1. Kill owned processes.
2. Append LOG.md row: `UNFINISHED / watchdog` with last known numbers and branch tip.
3. STATUS.md: mark task `UNFINISHED — check branch <name> before retry`.
4. Do **not** merge. Do **not** retry the same brief a third time if it already failed twice on merit; watchdog abandon ≠ merit FAIL unless numbers were measured worse.
5. Choose next highest gain/cost task (or REVIEWER+TUNER if two phases <0.5% gain).

---

## Recommended spawn brief addendum (future agents)

Append to every Task prompt:

```text
WATCHDOG: Soft [N] min / hard [M] min from spawn. If you cannot commit a PASS
or explicit FAIL report by hard timeout, stop processes and write a short
FAIL/UNFINISHED note on your branch (WATCHDOG.md) then exit. Orchestrator
cannot message you mid-flight — push progress commits if a step takes >15 min.
```
