# ORCH — P2-SPEED RESUME 2026-11-26T02:05Z
Continue PHASE2_SPEED_BITIDENTICAL.md after storage abandon.
- Worktree: /tmp/workspace/prometheus-p2-speed · branch agent/p2-speed
- CARGO_TARGET_DIR=/tmp/workspace/prometheus-p2-speed/target-p2-speed
- Tip archive sha256 must stay d400abbca2506ad3b5b5ae8353e71c7c1f386106e567340f3ac90f874adc60da (5680631 B)
- Uncommitted microopts already present — paired retime tip vs current tree first (GNU time, idle machine)
- Then ≤1 more serious opt attempt if CPU save unclear; document before/after; commit+push agent/p2-speed if KEEP-worthy speed
- Do NOT change prediction math / mixes / table sizes. No judge.sh edits for scoring.
- RAM=NO_CAP; label UNVERIFIED FOR 10 GB. Strip Cursor Co-authored-by.

## RESULT (P2-SPEED) — KEEP

- Bit-identical: YES (5680631 / sha256 d400abbca2506ad3b5b5ae8353e71c7c1f386106e567340f3ac90f874adc60da)
- slice25 paired means: tip WALL 124.90s RSS~3548MB → opt WALL 94.91s RSS~3548MB (−24% / −30s)
- CPU-h saved est.: ~0.033 enwik8; ~0.33–0.40 enwik9 projected
- RAM=NO_CAP UNVERIFIED FOR 10 GB
- Details: tune/P2_SPEED_RESULT.md
