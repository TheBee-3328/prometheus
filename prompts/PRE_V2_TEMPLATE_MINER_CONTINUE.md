# PREPROCESSOR-V2 — TEMPLATE-MINER trial (autonomous continuation)

**Role:** ORCHESTRATOR on the dedicated / continuation machine. Spawn agents / run judge; merge only on PASS.  
**Branch base:** pull latest `main`, work on `agent/pre-v2` (or recreate from `main` if missing).  
**Do not** auto-start enwik9 for the keep/throw gate. **enwik8 judge only** until templates are kept or discarded.  
**Dedicated server:** use this host for full-corpus mining (`enwik8` + full `enwik9`) and long judges; one heavy compress at a time.

## Goal

Finish **TEMPLATE-MINER**: measure structural compressed cost, mine a template dictionary, merge it with the static markup DICT in one tokenizer pass, and decide whether it earns SCORE. If not within a fixed budget, **throw it out** and **continue autonomous orch without templates** (normal DIAG / tuner) — do **not** stop the loop after THROW.

## Baseline (beat or match this)

Re-measure from current `main` / `STATUS.md` tip before starting (tip moves). As of handoff, post–DIAG-R6:

```
ROUNDTRIP=OK COMPRESSED=… DECOMPRESSOR=… SCORE=21583876
```

(Confirm exact line with `nix develop -c ./judge.sh enwik8 .` on clean `main` if STATUS disagrees.)

`SCORE = COMPRESSED + DECOMPRESSOR` (embedded template blob counts in DECOMPRESSOR). A stage **only keeps** if SCORE ≤ baseline (prefer strict improve; equal OK only with smaller/cleaner code).

## What already ran (do not redo blindly)

On a prior laptop session (`agent/pre-v2` WIP — **discarded / not on main**):

- **Step 1 (enwik8 only):** structure cost via `prometheus-diag --structure-stats` (full model + preprocessor). Approx compressed-byte share:
  - xml_wrapper ≈ 0.24%
  - entity_tag ≈ 0.48%
  - `{{templates}}` ≈ 14.9%
  - URLs ≈ 2.83%
  - **(a)+(b)+(c)+(d) ≈ 18.5%** → **above 3% gate → scheme may pay off; do not stop at Step 1**
- **enwik9 structure-stats** was started but **not finished** — **re-run on this server** and append both corpora to `DIAG.md`.
- Early template encode trial (64 mined skeletons, inline params) **regressed** SCORE (~21794083 vs then-baseline 21778410; COMPRESSED +13KB, DECOMPRESSOR +~2.4KB). Treat as a failed first attempt, not a final THROW — remine on post-capitals stream, prune by measured SCORE, try side-stream params.

## Rules (mandatory)

### Analysis (Step 1)

1. Extend / use `prometheus-diag --structure-stats` (cost-by-byte-class instrumentation + structural labels).  
2. Report total compressed-byte equivalents for: (a) XML page wrapper, (b) entity-escaped HTML tags, (c) `{{templates}}`, (d) URLs, (e) everything else — on **enwik8** and **full enwik9**. Put tables in `DIAG.md`.  
3. If (a)+(b)+(c)+(d) **under 3%** of total on both → stop template work, write gate FAIL in `DIAG.md`, then **continue orch without templates** (same as THROW path below).

### Mining (Step 2)

1. Implement / finish `tools/mine_templates.py`:
   - Tokenize structure (tags, entity-escaped tags, `{{...}}`, `[[...]]`, `[http...]`) vs variables (attr values, URLs, numbers, free text).  
   - Skeletons with placeholders; mine **largest first**; after accept, **cover spans** so nested/overlapping hits are not double-counted.  
   - Rank by **estimated net gain** = measured compressed cost of occurrences (from model per-byte cost log) **minus** dictionary cost — not raw byte savings.  
   - Emit `generated/template_dict.bin` (+ `.txt` / `.json` report).  
2. **Mine on the post-capitals stream** (templates run after capitals in the preprocessor). Capitals rewrite `>X` → `CAP_GT` + `x`; raw-file skeletons will miss.  
3. Prefer building dicts from **full enwik9** on this server; validate ranking with enwik8 cost log.  
4. Keep dict modest (e.g. ≤64–128 entries) unless SCORE math clearly wants more — DECOMPRESSOR grows with blob + decode code.

### Transform (Step 3)

1. Second stage **merged with static markup DICT into one greedy tokenizer pass**.  
2. Encoding: escape byte + 1–2 byte template indices; escape literal `0x80–0xFF` and the escape byte itself. Static markup may keep bare `0x80+` tokens if that preserves SCORE; do not gratuitously break the existing markup encoding.  
3. Parameters: **inline** after each template code **and** a **separate parameter stream**; keep whichever scores better on enwik8.  
4. `encode → decode` must equal original on all of enwik9 **and** adversarial inputs (valid/invalid UTF-8, empty, all bytes `0x00–0xFF`). Do **not** touch judge files.

### Keep gate (Step 4)

1. Keep each template / group only if judge **SCORE** (compressed + decompressor, including dictionary) improves on enwik8.  
2. Confirm keepers on enwik9 **only after** an enwik8 SCORE win (or at phase end if KEEP).  
3. Report gain per template group and final dictionary size in `DIAG.md`.

## Procedure

### Phase A — data + Step 1 on this server

1. `./scripts/reconstruct_data.sh both` if `enwik8`/`enwik9` missing.  
2. Build diag: `nix develop -c cargo build --release -p prometheus-diag`.  
3. Run structure-stats on **enwik8** and **full enwik9**; write results into `DIAG.md` (TEMPLATE-MINER section).  
4. Apply 3% gate. If fail → log and jump to Phase C **THROW** path (then continue orch).

### Phase B — mine full dicts + implement transform (≤ 15 cycles)

1. Implement miner + preprocessor templates stage on `agent/pre-v2`.  
2. Generate cost log(s) as needed (`prometheus-diag --cost-log …`); mine **enwik9** (and/or enwik8) post-capitals.  
3. Enable templates in `stages.rs` (`PRE_STAGE_TEMPLATES`); pick `PARAM_INLINE` true/false by measured SCORE.  
4. First templates-on judge: `nix develop -c ./judge.sh enwik8 .` → log full line in `LOG.md` / `DIAG.md`.  
5. If SCORE already ≤ baseline → **KEEP**; merge; continue normal orch.  
6. If worse → **tuning cycles** with templates still on (≤ **15** after first templates-on judge):
   - Prune templates by measured gain; try inline vs side-stream; shrink blob; small param/mixer tweaks that help the new byte distribution.  
   - Do **not** burn cycles on unrelated huge features.  
   - One **cycle** = one coherent change set + enwik8 judge (or equivalent measured SCORE).  
7. After each cycle: `DIAG.md` with SCORE Δ vs baseline and vs previous cycle.

### Phase C — keep or throw (then continue either way)

**Keep** if within ≤15 cycles `SCORE ≤` current `main` best:

- Leave templates on; commit `generated/template_dict.bin` + code; merge to `main` on PASS.  
- Note dict size, PARAM_INLINE choice, DECOMPRESSOR delta, per-group gains in `DIAG.md`.  
- Optional: confirm enwik9 once.  
- Continue orch with templates enabled.

**Throw** if after 15 cycles SCORE is still worse (or Step 1 gate fails):

- Set `PRE_STAGE_TEMPLATES = false`; stop embedding the blob in the default path (empty blob / dead code OK if unused).  
- Do **not** merge a regressing templates-on binary to `main`.  
- Log in `DIAG.md` + `LOG.md`: `TEMPLATE_MINER REJECT after 15 cycles; resume without templates`.  
- **Immediately continue autonomous orch without the template dict** (DIAG-R6 / model tuning / other preprocessor trials as in `STATUS.md`). **Do not stop** after THROW — THROW only ends the template trial, not the orchestration loop.

## Out of scope for this prompt

- Do **not** require enwik9 for the keep/throw gate (except optional confirm after KEEP).  
- Do **not** conflate with the **word/phrase dict** trial (`prompts/PRE_V2_WORD_DICT_CONTINUE.md`) — separate verdicts.  
- Do **not** retry abandoned briefs (P2-WORD model, P4-ORDERS-MIX).  
- Don’t touch `judge.sh` / judge contract files.

## Environment

- Always `nix develop -c …`.  
- Full enwik9 structure-stats + mining belong on this dedicated server (time/RAM).  
- 14 GB hosts: one heavy compress at a time.  
- Push work on `agent/pre-v2` (or task branch); merge to `main` only on judge PASS + SCORE rule above.

## Success report (end of trial)

Write a short block at the end of `DIAG.md`:

```
## PRE-V2 TEMPLATE-MINER trial
- Step 1 abcd % (enwik8 / enwik9): …
- Dict size / N templates: …
- PARAM_INLINE: true|false
- Cycles used: k/15
- Best SCORE with templates: …
- Baseline: …
- Verdict: KEEP | THROW
- Next: continue orch with|without templates (… )
```
