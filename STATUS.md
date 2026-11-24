# STATUS

**Current phase:** **PAUSED** — operator reloading into tmux (resume shortly)  
**Current best SCORE on main:** **28011126** (enwik8)  
**Best line (merged):** `ROUNDTRIP=OK COMPRESSED=27981374 DECOMPRESSOR=29752 SCORE=28011126`  
**Agent runs used:** 17 (informational; no auto-stop)  
**Stop policy:** Operator manual stop; budget auto-stop disabled.  

## PAUSE snapshot (2026-11-24 ~04:20 UTC+1)

Orch + subagent **compute stopped** for tmux reload. Worktrees kept.

### Unmerged win ready (do this first on resume)

| Candidate | Branch / path | SCORE | vs main |
|---|---|---|---|
| **R3_r102_only** | `agent/tuner-r3` worktree `/tmp/workspace/prometheus-tuner-r3` | **28010892** | **−234** (beats main) |

- Judge line: `ROUNDTRIP=OK COMPRESSED=27981140 DECOMPRESSOR=29752 SCORE=28010892 …`
- Params JSON: `tune/candidates/R3_r102_only.json`  
  `ORDER2_RESCALE=102`, `MIX_WEIGHT=[120,160,190,230,245]`, `STRUCT_MIX_WEIGHT=14`
- Validate out: `tune/validate_R3_r102_only.out`
- Branch had **no PASS commit yet** when paused (dirty tree; mid-validate loop killed)
- **Resume steps:** apply that JSON → `params.rs`, adversarial 10/10, commit on `agent/tuner-r3`, orch merge + push

### Other R3 validates (all worse or closer)

| Name | SCORE |
|---|---|
| R3_s15_r102 | 28011032 |
| R3_s15_r98 | 28011124 |
| R3_s15_only | 28011265 |
| R3_best | 28011499 |
| R3_m0_123 | 28011803 |

### Live agents at pause

- **TUNER-R3** (`e882527f-c0ab-4bbd-bc52-d97d9ae46959`) — interrupted mid multi-validate; do not resume mid-flight; finish winner manually or respawn with brief “commit R3_r102_only”
- **P5-REVIEW** — already merged
- **P5-SPARSE / TUNER-R2** — already merged

### Worktrees

```
/tmp/workspace/prometheus           main
/tmp/workspace/prometheus-tuner-r3  agent/tuner-r3   ← unmerged win
/tmp/workspace/prometheus-p5-sparse agent/p5-sparse-xml
/tmp/workspace/prometheus-p5-review agent/p5-review
/tmp/workspace/prometheus-tuner-r2  agent/tuner-r2
```

## Merges this session (before pause)

| Task | SCORE |
|---|---|
| TUNER-R2 | 28042709 |
| P5-SPARSE-XML | 28011126 |
| P5-REVIEW | docs @ 28011126 |
