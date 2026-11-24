# PREPROCESSOR-V2 — word/phrase dict trial (autonomous continuation)

**Role:** ORCHESTRATOR on the continuation machine. Spawn agents / run judge; merge only on PASS.  
**Branch base:** pull latest `main`, work on `agent/pre-v2` (or recreate from `main` if missing).  
**Do not** auto-start enwik9 for this trial. **enwik8 judge only** until the dict is kept or discarded.

**Companion:** Also run `prompts/PRE_V2_TEMPLATE_MINER_CONTINUE.md` on this machine. **Both trials must happen.** Separate keep/throw verdicts; THROW on one does **not** cancel the other. Prefer **word dict first**, then TEMPLATE-MINER (one heavy compress at a time). After each THROW or KEEP, **continue autonomous orch** — do not stop the loop.

## Goal

Decide whether the **XML-free word/phrase dictionary** stage earns its keep on SCORE. If not within a fixed budget, **throw it out** and resume normal model tuning without it (then proceed to / continue TEMPLATE-MINER and further orch).

## Baseline (beat or match this)

From current `main` / STATUS (re-measure if tip moved):

```
ROUNDTRIP=OK COMPRESSED=21546316 DECOMPRESSOR=37560 SCORE=21583876
```

`SCORE = COMPRESSED + DECOMPRESSOR` (embedded dict blob counts in DECOMPRESSOR). A stage **only keeps** if SCORE ≤ baseline (prefer strict improve; equal OK only with smaller/cleaner code).

## Dictionary rules (mandatory)

- This dict is **natural language only** — words **and** multi-word phrases OK.
- **No XML / dump markup** in this dict (no tags, entities, schema tokens like `page`/`revision`/`timestamp`). Markup stays in the separate static markup (and optional template) stage.
- Build with: `python3 tools/build_word_dict.py enwik9`  
  - Memory-capped two-pass; strips `<tags>` / `&entities;`.  
  - Emits `generated/word_dict_{4096,16384,49152}.bin` + report.  
  - Activate embed: `python3 tools/build_word_dict.py --select N` → `generated/word_dict.bin`.  
- Set `WORD_DICT_N` to match selected blob; flip `PRE_STAGE_WORDS` in `crates/prometheus-core/src/preprocessor/stages.rs`.  
- Reversible: encode→decode must equal input (unit tests + `tests/adversarial_roundtrip.sh`). Escape collisions with dict codes.

## Procedure

### Phase A — generate dicts

1. `./scripts/reconstruct_data.sh both` if `enwik8`/`enwik9` missing at repo root.  
2. Run `python3 tools/build_word_dict.py enwik9` (expect tens of minutes; do **not** OOM — script prunes).  
3. Record blob sizes from `generated/word_dict_report.md` (4k / 16k / 48k trade-off).  
4. Default trial size: **4096** first (`--select 4096`). Only try 16k/48k if 4k is close but DECOMPRESSOR math suggests a larger set could win.

### Phase B — enable words + tune (≤ 15 cycles)

1. Enable `PRE_STAGE_WORDS = true` (keep capitals + markup as on `main`; leave numeric off unless a prior measured win exists).  
2. `nix develop -c ./judge.sh enwik8 .` → log full SCORE line in `LOG.md` / `DIAG.md`.  
3. If SCORE already ≤ baseline → **KEEP** dict; merge branch; continue to TEMPLATE-MINER / normal orch. Stop this prompt’s discard path.  
4. If SCORE worse → run **tuning cycles** aimed at recovering SCORE **with the dict still on**:
   - Prefer param / mixer / soft-reset / SSE / APM / word-mix weight knobs (`params.rs`, small model mixes).  
   - Do **not** burn cycles on unrelated huge features.  
   - One **cycle** = one coherent change set + enwik8 judge (or equivalent measured SCORE).  
5. Hard budget: **15 cycles** after the first words-on judge.  
6. After each cycle: update `DIAG.md` with SCORE Δ vs baseline and vs previous cycle.

### Phase C — keep or throw (then continue)

**Keep** if within ≤15 cycles you reach `SCORE ≤ 21583876` (or current `main` best if newer):

- Leave `PRE_STAGE_WORDS = true`, commit selected `generated/word_dict.bin` (+ matching N), merge to `main` on PASS.  
- Note which N won and DECOMPRESSOR delta in `DIAG.md`.  
- **Continue** with TEMPLATE-MINER (`PRE_V2_TEMPLATE_MINER_CONTINUE.md`) and further orch.

**Throw** if after 15 cycles SCORE is still worse than baseline:

- Set `PRE_STAGE_WORDS = false` (and `WORD_DICT_N = 0` or leave unused).  
- Stop embedding the blob in the default path (empty/`word_dict_0` or don’t include in release path).  
- Do **not** merge a regressing words-on binary to `main`.  
- Log verdict in `DIAG.md` + `LOG.md`: `WORD_DICT REJECT after 15 cycles; resume without dict`.  
- **Continue autonomous orch without the word dict** — next: TEMPLATE-MINER trial, then DIAG/tuner as in `STATUS.md`. **Do not stop** after THROW.

## Out of scope for this prompt

- Do **not** require enwik9 confirmation for the keep/throw gate.  
- Do **not** retry abandoned briefs (P2-WORD model, P4-ORDERS-MIX).  
- Template/XML miner is the **companion** trial — run it too; don’t conflate verdicts.  
- Don’t touch `judge.sh` / judge contract files.

## Environment

- Always `nix develop -c …`.  
- 14 GB hosts: one heavy compress at a time; word-dict build alone is fine if capped.  
- Push work on `agent/pre-v2` (or task branch); merge to `main` only on judge PASS + SCORE rule above.

## Success report (end of trial)

Write a short block at the end of `DIAG.md`:

```
## PRE-V2 word dict trial
- N tried: …
- Cycles used: k/15
- Best SCORE with dict: …
- Baseline: …
- Verdict: KEEP | THROW
- Next: TEMPLATE-MINER / orch without word dict …
```
