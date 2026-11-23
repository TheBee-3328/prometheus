# Task brief template

Copy this block for every delegated agent. Fill every field.

```
TASK ID:
GOAL (one sentence):
OWNED FILES:
DO NOT TOUCH:
INTERFACE (from SPEC.md):
CURRENT BEST SCORE:
ACCEPTANCE TEST (exact command and pass condition):
BUDGET (time, tokens):
WHAT HAS ALREADY FAILED (from LOG.md):
```

## Rules (always attach)

- Work on your assigned git branch or worktree only.
- **Nix:** run all builds/tests via `nix develop -c …` (see `flake.nix`). Do not rely on host `~/.cargo`.
- Do not change judge files, data files, or scoring unless you are the JUDGE agent and the orchestrator authorized it.
- Integer / fixed-point only in the model path; no FP, no HashMap iteration dependence, no threads in the model path.
- Prefer small decompressor code: every byte counts toward SCORE.
- If stuck after two serious attempts, stop and report what you tried and the numbers.
- Final report: what changed, files touched, judge output before/after, next ideas.
