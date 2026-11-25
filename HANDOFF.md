# HANDOFF — 2026-11-25T20:18Z · Phase 2

**Best:** `CHKP-b5-20769768` — enwik8 SCORE **20769768** (B5-PROSE).  
**enwik9:** valid B5 re-judge **in flight** (bin 88192). Prior 186586969 run **INVALID** (stale 37k bin).

**Section 2:** mismatch explained; official time = 70000/T h (host T unknown); working ceilings 4h/dir + 9GB in `judge.sh`; webhook `\n` fixed; full determinism deferred until enwik9 frees RAM (2 MiB compress×2 hashes matched).

**Do not:** trust DECOMPRESSOR≠88192 enwik9 lines; auto-start extra compressors beside enwik9; use broken 20 054 334 archive.

**Reproduce enwik8:** `env -u CARGO_TARGET_DIR nix develop -c ./judge.sh enwik8 .`

**Next after enwik9:** record valid enwik9 + Resources on Discord; finish §2.4 determinism; profile B5; budget-ordered experiments.
