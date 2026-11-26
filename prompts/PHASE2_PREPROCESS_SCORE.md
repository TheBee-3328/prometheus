# Agent brief — Phase 2 Step 2: preprocessing (SCORE including decoder cost)

**Branch / worktree:** `agent/p2-pre` from B5 tip.  
**CARGO_TARGET_DIR:** local. **Nix:** mandatory.

## Goal
Improve **final SCORE** via reversible transforms (word dict growth/selection, caps, markup/templates, UTF-8/numbers). Decoder/aux bytes count in DECOMPRESSOR — net SCORE only.

## Context
- Word dict already embedded (`generated/word_dict.bin`); hit-gate + prose kept in B5.
- Prior template miner (C1) **THROW** — do not revive without a new slice proof.
- Ablation: turning WORD off costs ~**+86 KB** on 25 MiB → word path is valuable; grow carefully.

## Method
1. Slice-first (25 MiB). Prefer dict select/size and reversible markup that reduce CM entropy.
2. Every candidate: measure COMPRESSED Δ **and** stripped decomp (+aux) Δ → **SCORE Δ**.
3. Full enwik8 judge only if slice SCORE projection clears gate (orch: ≤ −15000 compressed on 25 MiB **or** clear SCORE win including decomp growth).
4. Adv 10/10 before merge.

## Forbidden
- Non-reversible transforms; external files at decompress not in `judge_aux` / embed.
- enwik9 auto.
- Ignoring decompressor size.

## Budget line
Report: SCORE bytes saved, +CPU_s/dir, +RSS, +DECOMPRESSOR; **bytes saved per CPU-hour** and **per GB**.
