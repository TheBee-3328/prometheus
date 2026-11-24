# DIAG — PROMETHEUS diagnose-and-fix

Branch: `agent/diag-fix`  
Agent: DIAG-FIX  
Start: 2026-11-24 (spawn)

## Architecture (as of baseline)

Existing components only:
- Preprocessor (cap fold + XML/wiki dict)
- Order-2 CM (byte o2 × bit path; u8 counters + rescale)
- Match model (LZ hash → expected bits; fixed MIX_WEIGHT / MATCH_CONF buckets)
- Sparse/XML structure CM (STRUCT_MIX when match inactive)
- 32-bit arithmetic coder, 16-bit p

**Missing (N/A for ablation disable):** orders 1/3/4/5/6, word model, adaptive mixer, APM/SSE, multi-layer mixer. These are findings if instrumentation supports adding them.

## Judge DECOMPRESSOR accounting (verified from JUDGE.md / judge.sh)

- `DECOMPRESSOR` = stripped `prometheus-decompress` byte size + optional `judge_aux.txt` paths
- Phase 0/1: no aux → exe only (no double-count of archive; compressor size NOT in SCORE)
- `SCORE = COMPRESSED + DECOMPRESSOR`
- `COMPRESSED` = archive file size on disk after compress

## STEP 1 — Baselines

### zstd floor (enwik8)

| Tool | Command | Size (bytes) | Wall time |
|------|---------|-------------:|----------:|
| zstd | `-19 --long=27` | **26395074** | ~32.5 s real |

**Merge floor:** nothing worse than zstd **26395074** compressed may merge.

### xz (enwik8)

| Tool | Command | Size (bytes) | Wall time |
|------|---------|-------------:|----------:|
| xz | `-9e` | *(running)* | *(running)* |

### PROMETHEUS judge (enwik8)

| Metric | Value |
|--------|------:|
| Judge line | *(running — see below)* |
| bpB | COMPRESSED×8/1e8 |

Claimed main baseline to verify:  
`ROUNDTRIP=OK COMPRESSED=27981374 DECOMPRESSOR=29752 SCORE=28011126` (~2.239 bpB)

---

## STEP 2 — Ablation (20 MB slice)

*(pending)*

## STEP 3 — Instrumentation

*(pending)*

## STEP 4 — Hypotheses

*(pending ranking from ablation/instrumentation)*

## STEP 5 — Fix log

| Fix | Hyp | Before SCORE | After SCORE | Keep? | Commit |
|-----|-----|-------------:|------------:|-------|--------|
| | | | | | |

## FINAL REPORT

*(incomplete)*
