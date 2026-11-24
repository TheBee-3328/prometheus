# TUNER-R3 — FAIL / invalidated

Paused mid-flight. Candidate `R3_r102_only` had recorded SCORE=28010892, but clean re-measure after resume:

`ROUNDTRIP=OK COMPRESSED=27983449 DECOMPRESSOR=29752 SCORE=28013201`

Worse than main 28011126. Likely cause: overlapping validate/judge runs sharing worktree state during the multi-candidate loop.

**Do not merge.** Main remains SCORE=28011126 (ORDER2_RESCALE=100).
