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
| xz | `-9e` | **24831656** | ~61.5 s real |

### PROMETHEUS judge (enwik8) — verified this worktree

```
ROUNDTRIP=OK COMPRESSED=27981374 DECOMPRESSOR=29768 SCORE=28011142 COMPRESS_SECONDS=32.63 DECOMPRESS_SECONDS=22.43 PEAK_RSS_MB=583
```

| Metric | Value |
|--------|------:|
| COMPRESSED | 27981374 |
| DECOMPRESSOR | 29768 (stripped exe; no judge_aux.txt) |
| SCORE | 28011142 |
| bpB | 2.23851 (COMPRESSED×8/1e8) |
| vs zstd floor | +1,586,300 bytes worse compressed |
| vs xz | +3,149,718 bytes worse compressed |
| vs claimed main | DECOMPRESSOR +16 vs 29752 (rebuild); COMPRESSED identical |

**Judge counting check:** COMPRESSED = archive `wc -c`; DECOMPRESSOR = exe only (no aux). No double-count. Compressor size excluded from SCORE (per JUDGE.md).

---

## STEP 2 — Ablation (20 MB slice = first 20e6 enwik8 bytes)

Tool: `crates/prometheus-diag` (debug only; not in SCORE path).  
PRE shrinks 20e6 → 18805754 (−6.0%).

| Component disabled | COMPRESSED | Δ vs full | Gain of component | Flag |
|--------------------|----------:|----------:|------------------:|------|
| **full** (all on) | **5746507** | 0 | — | — |
| no-pre | 5751634 | +5127 | **+0.09%** | SUSPECT (near-zero; wiki dict should help more) |
| no-match | 7475125 | +1728618 | **+30.08%** | OK (match healthy) |
| no-struct | 5754557 | +8050 | **+0.14%** | trivial STRUCT_MIX=14 |
| order2-only | 7437808 | +1691301 | — | o2 alone |
| match-only | 13105961 | +7359454 | — | match needs o2 |

**N/A (not in binary):** order-1, order-3/4/5/6, word model, adaptive mixer, APM/SSE layers.

Note: `no-match` (o2+struct=7475125) **worse** than `order2-only` (7437808) → struct alone hurts; only helps lightly when match gates it (`full` vs `no-struct`).

## STEP 3 — Instrumentation (same 20 MB, mode=full --stats)

| Stat | Value |
|------|------:|
| match_active_bits | 105766386 / 150446032 = **70.30%** |
| avg match_len (active) | **11.80** |
| match_hit_bytes | 16020175 / 18805754 ≈ **85.18%** |
| p pinned at 1 / 65534 | **0 / 0** |
| stretch/squash clamp | N/A (no stretch/squash in path) |
| mixer weights | N/A (fixed MIX_WEIGHT / STRUCT_MIX only) |
| hash | HASH_BITS=26, overwrite replace, no checksum |

Avg cost by byte class (approx mbits/bit; relative only — estimator coarse):

| Class | avg_mbits/bit | Notes |
|-------|-------------:|-------|
| bracket | 46582 | easiest |
| lt | 52801 | easy |
| gt | 148507 | |
| alpha | 194069 | bulk of bits |
| hibyte | 207397 | dict tokens |
| entity | 211078 | |
| other | 305792 | |
| digit | 333881 | expensive |
| **ws** | **442293** | **hardest** — needs better contexts |

## STEP 4 — Hypotheses (ranked by steps 2–3)

| Rank | Hyp | Evidence | Action |
|-----:|-----|----------|--------|
| 1 | **g** Missing contexts (o1/o3/o4/word) | Only o2 exists; ws/digit costly; 2.24 bpB far from lpaq | **Add o1 + hashed o3/o4** |
| 2 | **b** No adaptive mixer / selector | Fixed mix only; finding not a bug | Implement integer mixer after multi-ctx |
| 3 | **i** No SSE/APM | Absent | Add after mixer |
| 4 | **f** Match fire/conf | 70% active, +30% size — healthy; MATCH_CONF all 32700 may overconf | Tune after contexts |
| 5 | **h** Preprocessor weak | +0.09% only | Keep; improve dict later or disable if regresses |
| 6 | **e** Counter adaptation | u8 + RESCALE=100 | Try after contexts |
| 7 | **a** Hash too small / no checksum | Hit rate high @26b; low priority | Optional checksum |
| 8 | **c/d** p precision / stretch | Already 16-bit p; no stretch path | Add with mixer |
| 9 | **j** Time headroom | 32s compress, 583 MB RSS | Room for more tables |
| 10 | **k** Judge counting | Verified OK | Rejected |

## STEP 5 — Fix log

| Fix | Hyp | Before SCORE | After SCORE | Keep? | Commit |
|-----|-----|-------------:|------------:|-------|--------|
| o1 alone (mix=8) | g | 28011142 | **28007791** (C=27977623) | measured; folded into next | — |
| **o1+hashed o3/o4** (w=8/160/48, bits 22/23) | g | 28011142 | **26380505** | **KEEP** | `1d9837d` |
| broken adaptive mixer + o6 | b/g | — | slice 7.98e6 (worse) | **REJECT** | — |
| o6 fixed mix any weight | g | — | slice worse vs w6=0 | **REJECT** | — |
| **bigger o3/o4 tables** (bits 23/24) | a/j | 26380505 | **26143788** | **KEEP** | `29097b6` |
| **SSE layer** (mix=224) | i | 26143788 | **25548485** | **KEEP** | `b1eb4ae` |
| MATCH_CONF graduated | f | — | slice worse | **REJECT** (max conf best) | — |
| **word model** (bits=22, mix=64) | g | 25548485 | **25300821** | **KEEP** | `bde3516` |
| **SSE+match-bucket cxt** | i | 25300821 | **24476305** | **KEEP** | `e12b068` |
| **SSE2 byte-class** | i | 24476305 | **24235975** | **KEEP** | `4e6866a` |
| **tables 24/25 + word23 + rescale250** | a/e/j | 24235975 | **23754487** | **KEEP** | *(pending)* |

### Fix detail: add order-1 + hashed order-3/4

Judge after:
```
ROUNDTRIP=OK COMPRESSED=26350257 DECOMPRESSOR=30248 SCORE=26380505 COMPRESS_SECONDS=71.26 DECOMPRESS_SECONDS=65.80 PEAK_RSS_MB=607
```

| | Before | After | Δ |
|--|-------:|------:|--:|
| COMPRESSED | 27981374 | 26350257 | **-1631117** |
| DECOMPRESSOR | 29768 | 30248 | +480 |
| SCORE | 28011142 | 26380505 | **-1630637** |
| bpB | 2.23851 | 2.10802 | |
| vs zstd 26395074 | worse | **beats by 44817** | |

20 MB slice: 5746507 → 5475423 (−4.72%).

### Fix detail: larger o3/o4 hash tables (23/24 bits)

```
ROUNDTRIP=OK COMPRESSED=26113540 DECOMPRESSOR=30248 SCORE=26143788 COMPRESS_SECONDS=81.92 DECOMPRESS_SECONDS=77.22 PEAK_RSS_MB=632
```

| | Before | After | Δ |
|--|-------:|------:|--:|
| COMPRESSED | 26350257 | 26113540 | **-236717** |
| SCORE | 26380505 | 26143788 | **-236717** |
| bpB | 2.10802 | 2.08908 | |
| vs zstd | beats | beats by 281534 | |

20 MB slice: 5475423 → 5439385 (−0.66%).

### Fix detail: SSE (secondary symbol estimation)

```
ROUNDTRIP=OK COMPRESSED=25517693 DECOMPRESSOR=30792 SCORE=25548485 COMPRESS_SECONDS=93.54 DECOMPRESS_SECONDS=85.22 PEAK_RSS_MB=631
```

| | Before | After | Δ |
|--|-------:|------:|--:|
| COMPRESSED | 26113540 | 25517693 | **-595847** |
| SCORE | 26143788 | 25548485 | **-595303** |
| bpB | 2.08908 | 2.04142 | |

20 MB slice: 5439385 → 5260907 (−3.28%).

### Fix detail: word model

```
ROUNDTRIP=OK COMPRESSED=25269357 DECOMPRESSOR=31464 SCORE=25300821 COMPRESS_SECONDS=101.74 DECOMPRESS_SECONDS=96.27 PEAK_RSS_MB=639
```

| | Before | After | Δ |
|--|-------:|------:|--:|
| COMPRESSED | 25517693 | 25269357 | **-248336** |
| SCORE | 25548485 | 25300821 | **-247664** |
| bpB | 2.04142 | 2.02155 | |

20 MB slice: 5260907 → 5210195 (−0.96%).

### Fix detail: SSE match-length context

```
ROUNDTRIP=OK COMPRESSED=24444761 DECOMPRESSOR=31544 SCORE=24476305 COMPRESS_SECONDS=107.88 DECOMPRESS_SECONDS=103.58 PEAK_RSS_MB=640
```

| | Before | After | Δ |
|--|-------:|------:|--:|
| COMPRESSED | 25269357 | 24444761 | **-824596** |
| SCORE | 25300821 | 24476305 | **-824516** |
| bpB | 2.02155 | 1.95558 | |
| vs xz 24831656 | worse | **beats by 386895** | |

20 MB slice: 5210195 → 5032793 (−3.41%).

## Re-ablation note (after 3 keeps; pre-word)

Diag tool modes `no-match`/`no-struct`/`order2-only` still bypass o3/o4/SSE/word — numbers not comparable to `full=5260907`. Caveat recorded; not used for merge decisions.

## FINAL REPORT

*(in progress)*

### Fix detail: SSE2 byte-class context

```
ROUNDTRIP=OK COMPRESSED=24203663 DECOMPRESSOR=32312 SCORE=24235975 COMPRESS_SECONDS=116.87 DECOMPRESS_SECONDS=110.47 PEAK_RSS_MB=640
```

| | Before | After | Δ |
|--|-------:|------:|--:|
| COMPRESSED | 24444761 | 24203663 | **-241098** |
| SCORE | 24476305 | 24235975 | **-240330** |
| bpB | 1.95558 | 1.93629 | |

20 MB slice: 5032793 → 4975376 (−1.14%).

### Fix detail: larger tables + WORD_BITS=23 + RESCALE=250

```
ROUNDTRIP=OK COMPRESSED=23722175 DECOMPRESSOR=32312 SCORE=23754487 COMPRESS_SECONDS=122.91 DECOMPRESS_SECONDS=118.71 PEAK_RSS_MB=696
```

| | Before | After | Δ |
|--|-------:|------:|--:|
| COMPRESSED | 24203663 | 23722175 | **-481488** |
| SCORE | 24235975 | 23754487 | **-481488** |
| bpB | 1.93629 | 1.89777 | |

20 MB slice: 4975376 → 4885161 (−1.81%).

### Fix detail: order-0 + match hash fingerprint

```
ROUNDTRIP=OK COMPRESSED=23599584 DECOMPRESSOR=32872 SCORE=23632456 COMPRESS_SECONDS=129.43 DECOMPRESS_SECONDS=122.58 PEAK_RSS_MB=824
```

| | Before | After | Δ |
|--|-------:|------:|--:|
| COMPRESSED | 23722175 | 23599584 | **-122591** |
| SCORE | 23754487 | 23632456 | **-122031** |
| bpB | 1.89777 | 1.88797 | |

Checksum alone size-neutral on slice; order0 mix=80 carries the gain.

### Fix detail: order-5 + STRUCT_MIX=24

```
ROUNDTRIP=OK COMPRESSED=23137005 DECOMPRESSOR=32936 SCORE=23169941 COMPRESS_SECONDS=165.15 DECOMPRESS_SECONDS=158.01 PEAK_RSS_MB=856
```

| | Before | After | Δ |
|--|-------:|------:|--:|
| COMPRESSED | 23599584 | 23137005 | **-462579** |
| SCORE | 23632456 | 23169941 | **-462515** |
| bpB | 1.88797 | 1.85096 | |

MIX_WEIGHT / MIN_MIX_LEN sweeps: current best (reject changes).

### Fix detail: order-6 (mix=32) with SSE stack

```
ROUNDTRIP=OK COMPRESSED=23036149 DECOMPRESSOR=33016 SCORE=23069165 COMPRESS_SECONDS=192.52 DECOMPRESS_SECONDS=185.69 PEAK_RSS_MB=888
```

| | Before | After | Δ |
|--|-------:|------:|--:|
| COMPRESSED | 23137005 | 23036149 | **-100856** |
| SCORE | 23169941 | 23069165 | **-100776** |
| bpB | 1.85096 | 1.84289 | |

Note: earlier o6 without SSE was rejected; with SSE it helps.

## FINAL REPORT

### Before / after

| | Judge line | bpB |
|--|---|---:|
| **Baseline (start)** | `ROUNDTRIP=OK COMPRESSED=27981374 DECOMPRESSOR=29768 SCORE=28011142` | 2.23851 |
| **Best (end)** | `ROUNDTRIP=OK COMPRESSED=23036149 DECOMPRESSOR=33016 SCORE=23069165` | 1.84289 |

SCORE Δ: **−4,941,977** (−17.6%). Beats zstd floor (26395074) and xz (24831656).

### Floors

| Tool | Size | Time |
|------|-----:|-----:|
| zstd -19 --long=27 | 26395074 | ~32.5s |
| xz -9e | 24831656 | ~61.5s |
| PROMETHEUS best | **23036149** | ~193s compress / ~186s decompress |

### Ranked causes (measured SCORE gains)

| Rank | Cause | Approx SCORE gain | Commit |
|-----:|-------|------------------:|--------|
| 1 | Missing o1+o3+o4 contexts | −1,630,637 | `1d9837d` |
| 2 | SSE match-bucket context | −824,516 | `e12b068` |
| 3 | SSE primary layer | −595,303 | `b1eb4ae` |
| 4 | Larger tables + rescale 250 | −481,488 | `37ba74e` |
| 5 | Order-5 + STRUCT=24 | −462,515 | `4c2e6b5` |
| 6 | Word model | −247,664 | `bde3516` |
| 7 | SSE2 byte-class | −240,330 | `4e6866a` |
| 8 | Bigger o3/o4 (23/24) | −236,717 | `29097b6` |
| 9 | Order-0 + match fingerprint | −122,031 | `6aecd02` |
| 10 | Order-6 (with SSE) | −100,776 | *(this)* |

### Rejected with evidence

| Hyp | Evidence |
|-----|----------|
| Adaptive logistic mixer (naive) | Slice 7.98e6 vs 5.47e6 |
| o6 without SSE | Any weight worse on slice |
| MATCH_CONF graduated | All worse than max-32700 |
| MIX_WEIGHT retune | Current best on slice |
| MIN_MIX_LEN ≠ 6 | 7/8 worse; 4/5 noop (HASH_ORDER=6) |
| Judge double-count | Verified: SCORE=C+exe, no aux |

### Remaining suspects / unexplained (~2.54 MB to 20.5e6 target)

1. **No working adaptive mixer / APM chain** — fixed linear blends leave correlation on the table (lpaq/paq8/cmix gap).
2. **u8 counters** — saturate; u16 + state maps (paq) unused.
3. **Preprocessor weak** (+0.09% early) — dict/cap still shallow vs wiki-aware transforms.
4. **Match model** — single hash chain, no multi-candidate; conf still binary-max.
5. **Missing record/indirect/sparse-gap models** common in cmix-class compressors.
6. Soft time: compress already ~193s; room for heavier models before 20min if optimized.

### enwik9 projection

Scale ×10 on COMPRESSED ≈ **230 MB**; time ×10–12 ≈ **30–40 min** compress (above typical 20 min mid-weight budget unless sped up). RSS ~0.9 GB → may need ~1–2 GB on enwik9.

### Merge readiness

- SCORE **23069165** ≪ baseline **28011126** — ready on SCORE.
- Beats zstd/xz floors.
- Adversarial: **10/10 PASS** (at o5; re-verify after o6).
- Target 20.5e6 **not** reached — **partial** vs stop criterion.

---

# DIAG-R2

Branch: `agent/diag-r2`  
Agent: DIAG-R2  
Start: 2026-11-24  
Baseline to beat: `ROUNDTRIP=OK COMPRESSED=23036149 DECOMPRESSOR=33016 SCORE=23069165`

## Plan

1. Careful APM (paq-style stretch/squash + slow integer update) after SSE2 — **not** the rejected naive logistic mixer.
2. If APM stalls: tiny-LR context-selected mixer on 2–3 pre-blended signals.
3. If both regress: u16 counters on high-order CMs / APM-only path.

## Slice baseline (first 20e6 enwik8)

`COMPRESSED=4764551` (release compress, ~39s).

## Attempt 1 — paq-style APM after SSE2 (KEEP)

Design (careful, not naive mixer):
- `apm.rs`: stretch/squash + interpolated 33-bin APM, rate_shift=7
- Context: `last_byte × match_bucket` (2048) — distinct from SSE bit_cxt / SSE2 class
- Blend `APM_MIX_WEIGHT=224` into post-SSE2 p
- Fixed CM/match blends unchanged

| | Slice COMPRESSED | Δ |
|--|--:|--:|
| baseline | 4764551 | |
| +APM | **4708020** | **−56531 (−1.19%)** |

Full enwik8 judge:
```
ROUNDTRIP=OK COMPRESSED=22766317 DECOMPRESSOR=42552 SCORE=22808869 COMPRESS_SECONDS=206.63 DECOMPRESS_SECONDS=201.26 PEAK_RSS_MB=888
```

| | Before | After | Δ |
|--|--:|--:|--:|
| COMPRESSED | 23036149 | 22766317 | **−269832** |
| DECOMPRESSOR | 33016 | 42552 | +9536 (mostly STRETCH LUT) |
| SCORE | 23069165 | **22808869** | **−260296** |
| bpB | 1.84289 | 1.82131 | |

vs 20.5e6: gap ≈ 2.27 MB (was 2.54 MB).

Adversarial: **10/10 PASS**. Pushed `669042b`.

## Attempt 2a — APM2 class×bit_cxt (REJECT)

Chained second APM after APM1. Slice **4710362** vs APM1 **4708020** (**+2342**). Reverted.

## Attempt 2b — tiny 2-input logistic mixer (REJECT)

CM↔match/struct only, 64 sets (match_bucket×bit_pos), rate=2, init≈fixed MIX/STRUCT.
Slice **4878192** vs APM1 **4708020** (**+170172**, ~+3.6%). Reverted.
→ Two mixer-path regressions; switch to **u16 counters** on hashed orders.

## Attempt 3 — u16 hash-order counters (REJECT)

`HashOrderModel` n0/n1 → u16; `HASH_ORDER_RESCALE` sweep 400–3072 all **worse** than u8+APM1 (best delta +1716 at 400). Reverted.

## Attempt 4 — richer APM context (KEEP)

Expand APM cxt to `last_byte × bit_pos × match_bucket` (16384).
Slice **4668167** vs APM1 **4708020** (**−39853**).

```
ROUNDTRIP=OK COMPRESSED=22567638 DECOMPRESSOR=42568 SCORE=22610206 COMPRESS_SECONDS=217.53 DECOMPRESS_SECONDS=207.27 PEAK_RSS_MB=889
```

| | Before (APM1) | After | Δ |
|--|--:|--:|--:|
| COMPRESSED | 22766317 | 22567638 | **−198679** |
| DECOMPRESSOR | 42552 | 42568 | +16 |
| SCORE | 22808869 | **22610206** | **−198663** |

vs post-DIAG-FIX baseline SCORE 23069165: **−458959**. Gap to 20.5e6 ≈ 2.07 MB.

## Attempt 5 — hashed order-7 mix=16 (KEEP)

Slice (mix sweep): best **4663521** at MIX=16 (vs rich 4668167, **−4646**).

```
ROUNDTRIP=OK COMPRESSED=22531741 DECOMPRESSOR=42760 SCORE=22574501 COMPRESS_SECONDS=260.00 DECOMPRESS_SECONDS=256.14 PEAK_RSS_MB=905
```

| | Before (rich APM) | After | Δ |
|--|--:|--:|--:|
| COMPRESSED | 22567638 | 22531741 | **−35897** |
| DECOMPRESSOR | 42568 | 42760 | +192 |
| SCORE | 22610206 | **22574501** | **−35705** |

Gap to 20.5e6 ≈ 2.03 MB.

## Attempt 6 — paq-style SSE cell updates (REJECT)

Replace Laplace n0/n1 in SSE/SSE2 with single-u16 `(tgt-cur)>>6` updates.
Slice **4731280** vs o7 **4663521** (**+67759**). Reverted.

APM mix/rate sweep: current 224/7 near-optimal (best alt MIX=192 RATE=6 only −63 B on slice).

## FINAL REPORT (DIAG-R2)

### Verdict: **partial**

Clear SCORE win vs post-DIAG-FIX baseline; **≤20.5e6 COMPRESSED not reached**.

### Best judge line

```
ROUNDTRIP=OK COMPRESSED=22531741 DECOMPRESSOR=42760 SCORE=22574501 COMPRESS_SECONDS=260.00 DECOMPRESS_SECONDS=256.14 PEAK_RSS_MB=905
```

| | Baseline | Best | Δ |
|--|--:|--:|--:|
| COMPRESSED | 23036149 | **22531741** | **−504408** |
| DECOMPRESSOR | 33016 | 42760 | +9744 |
| SCORE | 23069165 | **22574501** | **−494664** |
| bpB | 1.84289 | 1.80254 | |

vs zstd 26395074 / xz 24831656: still ahead. Gap to 20.5e6: **≈2.03 MB**.

### Commits (agent/diag-r2)

| Commit | Change | SCORE |
|--------|--------|------:|
| `669042b` | paq-style APM after SSE2 | 22808869 |
| `e18070b` | richer APM cxt (byte×bit_pos×match) | 22610206 |
| `35d2a63` | hashed order-7 mix=16 | **22574501** |

### Acceptance

- Judge ROUNDTRIP=OK and SCORE < 23069165: **YES**
- ≤20.5e6 COMPRESSED: **NO**
- Adversarial: **10/10 PASS** (re-verified clean; one false FAIL was concurrent rebuild race)

### Rejected

| Hyp | Evidence |
|-----|----------|
| APM2 class×bit_cxt | slice +2342 |
| Tiny 2-input logistic mixer | slice +170172 |
| u16 hash-order counters | all rescale worse |
| paq-style SSE updates | slice +67759 |
| APM mix/rate retune | ≤63 B, not worth judge |

### Next ideas (toward 20.5e6)

1. Multi-candidate / longer match + soft conf (not binary max).
2. Indirect / record / sparse-gap models (cmix-class).
3. Runtime-built STRETCH (reclaim ~8 KB DECOMPRESSOR) + more APM contexts (order-1×class).
4. State-map / dual-rate counters (paq8) instead of plain u16 rescale.
5. Deeper preprocessor (wiki dict / entity / UTF-8) — early ablation was weak but may compound now.
6. Speed: compress ~260s; optimize before heavier models for enwik9.

# DIAG-R3

Branch: `agent/diag-r3`  
Agent: DIAG-R3  
Start: 2026-11-24  
Baseline to beat: `ROUNDTRIP=OK COMPRESSED=22531741 DECOMPRESSOR=42760 SCORE=22574501`

## Plan

1. Multi-candidate match (2–4 chain; longest verified) — priority.
2. Indirect / record model (integer only).
3. State-map / bit-history (not blanket u16 hash counters).
4. STRETCH LUT reclaim if SCORE-neutral/positive.
5. Preprocessor deepen only if ablation shows gain.

## Slice baseline (first 20 MiB enwik8, tune profile)

`COMPRESSED=4884107`.

## Attempt 1 — multi-candidate match chain (KEEP)

Design: 4-deep hash-slot chain (`ht` + 3×`ht_prev`). On match start, verify all candidates; pick longest verified context (≥HASH_ORDER). Primary keeps fingerprint reject; chain slots byte-verify only. Confidence/mix still driven by match length.

| Candidates | Slice COMPRESSED | Δ vs base |
|-----------:|-----------------:|----------:|
| 1 (base) | 4884107 | |
| 2 | 4877078 | −7029 |
| 3 | 4874056 | −10051 |
| **4** | **4872291** | **−11816** |

Full enwik8 judge (cand=4):
```
ROUNDTRIP=OK COMPRESSED=22439085 DECOMPRESSOR=43352 SCORE=22482437 COMPRESS_SECONDS=281.75 DECOMPRESS_SECONDS=273.51 PEAK_RSS_MB=1673
```

| | Before | After | Δ |
|--|--:|--:|--:|
| COMPRESSED | 22531741 | 22439085 | **−92656** |
| DECOMPRESSOR | 42760 | 43352 | +592 |
| SCORE | 22574501 | **22482437** | **−92064** |

Gap to 20.5e6 ≈ 1.94 MB (was 2.03 MB). Adversarial: **10/10 PASS**.

## Attempt 2 — indirect / record model (KEEP)

Design: `IndirectModel` — `t1[256]` / `t2[65536]` last-byte-after o1/o2; hashed `(bit_cxt × prev0 × ind1 × ind2)` counters (`INDIRECT_BITS=24`); blend `INDIRECT_MIX_WEIGHT` into CM before match.

Slice vs multi-cand=4 (4872291):

| MIX | Slice COMPRESSED | Δ |
|----:|-----------------:|--:|
| 24 | 4800899 | −71392 |
| 32 | 4789700 | −82591 |
| 36 | 4786839 | −85452 |
| **40** | **4785692** | **−86600** |
| 44 | 4786179 | −86112 |
| 48 | 4788459 | −83832 |
| 56 | 4797719 | −74572 |

Full enwik8 (MIX=40):
```
ROUNDTRIP=OK COMPRESSED=22037144 DECOMPRESSOR=44168 SCORE=22081312 COMPRESS_SECONDS=327.34 DECOMPRESS_SECONDS=309.37 PEAK_RSS_MB=1705
```

| | Before (mc×4) | After | Δ |
|--|--:|--:|--:|
| COMPRESSED | 22439085 | 22037144 | **−401941** |
| DECOMPRESSOR | 43352 | 44168 | +816 |
| SCORE | 22482437 | **22081312** | **−401125** |

Gap to 20.5e6 ≈ 1.54 MB.

## Attempt 3 — runtime heap STRETCH LUT (KEEP)

Replace compile-time `static STRETCH[4096]` with `OnceLock<Box<[i16]>>` built once via the same binary-search. Slice COMPRESSED unchanged (4785692).

Full enwik8:
```
ROUNDTRIP=OK COMPRESSED=22039256 DECOMPRESSOR=36680 SCORE=22075936 COMPRESS_SECONDS=321.50 DECOMPRESS_SECONDS=311.63 PEAK_RSS_MB=1705
```

| | Before (indirect) | After | Δ |
|--|--:|--:|--:|
| COMPRESSED | 22037144 | 22039256 | +2112 (stretch path drift) |
| DECOMPRESSOR | 44168 | 36680 | **−7488** |
| SCORE | 22081312 | **22075936** | **−5376** |

## Attempt 4 — sparse-gap model (REJECT)

`GapModel` hashed (hist[0,1,2,4] × bit_cxt), GAP_BITS=22.
Slice vs 4785692: MIX=32 → **4811629 (+25937)**; MIX=16 → **4786423 (+731)**. Reverted.

## FINAL REPORT (DIAG-R3)

### Verdict: **partial**

Clear SCORE wins vs DIAG-R2 baseline; **≤20.5e6 COMPRESSED not reached**.

### Best judge line

```
ROUNDTRIP=OK COMPRESSED=22039256 DECOMPRESSOR=36680 SCORE=22075936 COMPRESS_SECONDS=321.50 DECOMPRESS_SECONDS=311.63 PEAK_RSS_MB=1705
```

| | Baseline (R2) | Best (R3) | Δ |
|--|--:|--:|--:|
| COMPRESSED | 22531741 | **22039256** | **−492485** |
| DECOMPRESSOR | 42760 | 36680 | −6080 |
| SCORE | 22574501 | **22075936** | **−498565** |
| bpB | 1.80254 | 1.76314 | |

Gap to 20.5e6: **≈1.54 MB** (was 2.03 MB).

### Commits (agent/diag-r3)

| Commit | Change | SCORE |
|--------|--------|------:|
| `cda121b` | multi-candidate match chain×4 | 22482437 |
| `6eecaa2` | indirect/record model MIX=40 | 22081312 |
| `5f645c4` | heap-built STRETCH LUT | **22075936** |

### Acceptance

- Judge ROUNDTRIP=OK and SCORE < 22574501: **YES**
- ≤20.5e6 COMPRESSED: **NO**
- Adversarial: **10/10 PASS** on each kept commit

### Rejected

| Hyp | Evidence |
|-----|----------|
| Sparse-gap MIX=32/16 | slice +25937 / +731 |

### Next ideas (toward 20.5e6)

1. State-map / dual-rate bit-history (paq8) on selected contexts — not tried this round.
2. Match secondary hash (order-4 / alt salt) beyond chain depth.
3. Deeper indirect (order-3 last-byte table) or match×indirect cross context.
4. Fix STRETCH drift (+2 KB COMPRESSED) — restore bit-identical stretch vs static LUT.
5. Preprocessor deepen only with fresh ablation under current ensemble.
6. Speed: compress ~320s / RSS ~1.7 GB — trim match chain memory before enwik9.

# DIAG-R4

Branch: `agent/diag-r4`  
Agent: DIAG-R4  
Start: 2026-11-24  
Baseline to beat: `ROUNDTRIP=OK COMPRESSED=22039256 DECOMPRESSOR=36680 SCORE=22075936`

## Plan

1. State-map / dual-rate bit-history (not blanket u16 hash counters) — priority.
2. Match secondary hash beyond chain.
3. Deeper indirect / match×indirect cross.
4. Fix STRETCH drift if cheap.
5. RSS trim for match chain if SCORE-neutral.

## Attempt 1 — Order-2 count-dependent StateMap (KEEP)

Design: replace Order-2 Laplace n0/n1 with paq-style packed StateMap cells (`p<<16|n`), update `p += ((tgt-p)*3)/(n+4)`. Dual-rate via count-dependent step (not fixed-shift dual p, which regressed +910 KB on slice). Not hashed-order u16 counters.

Rejected precursors (same attempt, reverted before judge):
- Fixed dual-rate p_fast/p_slow replacing Laplace: slice +910 KB.
- Global bit-history StateProb mix into Laplace: slice gains looked large but deferred (shared 256-state map).

Slice (25 MiB @ offset 8M): Laplace≈5695274 → StateMap **5686155** (−9119).

Full enwik8:
```
ROUNDTRIP=OK COMPRESSED=22008866 DECOMPRESSOR=36744 SCORE=22045610 COMPRESS_SECONDS=315.71 DECOMPRESS_SECONDS=329.03 PEAK_RSS_MB=1756
```

| | Before | After | Δ |
|--|--:|--:|--:|
| COMPRESSED | 22039256 | 22008866 | **−30390** |
| DECOMPRESSOR | 36680 | 36744 | +64 |
| SCORE | 22075936 | **22045610** | **−30326** |

Gap to 20.5e6 ≈ 1.51 MB. Adversarial: **10/10 PASS**.

## Attempt 2 — match secondary hash order-4 (REJECT)

Separate `ht2` (HASH2_BITS=24) with order-4 alt-salt. Loose bar (accept ≥4): slice **+80910**. Strict bar (≥HASH_ORDER=6): slice **+24** (neutral). Reverted.

## Attempt 3 — indirect order-3 last-byte table (KEEP)

Add `t3[65536]` last-byte-after hashed o3; fold `ind3` into indirect hash. MIX=40 still best (32/48 worse on slice).

Slice vs StateMap-o2 base 5686155: **5669982** (−16173).

Also tried t3+StateMap-on-indirect cells: slice +8282 — rejected combo.

Full enwik8:
```
ROUNDTRIP=OK COMPRESSED=21933188 DECOMPRESSOR=36952 SCORE=21970140 COMPRESS_SECONDS=342.13 DECOMPRESS_SECONDS=354.65 PEAK_RSS_MB=1756
```

| | Before (SM-o2) | After | Δ |
|--|--:|--:|--:|
| COMPRESSED | 22008866 | 21933188 | **−75678** |
| DECOMPRESSOR | 36744 | 36952 | +208 |
| SCORE | 22045610 | **21970140** | **−75470** |

Gap to 20.5e6 ≈ 1.43 MB.

## Attempt 4 — StateMap Order-1 (REJECT)

Same p+n StateMap on Order-1: slice **−3** (noise). Reverted.

## Attempt 5 — match×indirect cross StateMap (KEEP)

`CROSS_SIZE=32768` cells: `match_bucket × ind1 × bit_pos × expected_bit`. Blend `MATCH_IND_CROSS_WEIGHT` into CM before match/struct.

Slice vs t3 base 5669982:

| CROSS | Slice | Δ |
|------:|------:|--:|
| 0 | 5669982 | |
| **16** | **5665469** | **−4513** |
| 24 | 5666217 | −3765 |
| 32 | 5669294 | −688 |
| 48 | 5684609 | +14627 |

Full enwik8 (CROSS=16):
```
ROUNDTRIP=OK COMPRESSED=21918203 DECOMPRESSOR=37288 SCORE=21955491 COMPRESS_SECONDS=346.11 DECOMPRESS_SECONDS=341.16 PEAK_RSS_MB=1756
```

| | Before (t3) | After | Δ |
|--|--:|--:|--:|
| COMPRESSED | 21933188 | 21918203 | **−14985** |
| DECOMPRESSOR | 36952 | 37288 | +336 |
| SCORE | 21970140 | **21955491** | **−14649** |

Gap to 20.5e6 ≈ 1.42 MB.

## Attempt 6 — StateMap soft-reset at n=255→208 (KEEP)

When count saturates, set n=208 instead of sticking at 255 so nonstationary contexts re-adapt. Affects Order-2 + match×indirect cross cells.

Slice: **5664646** vs 5665469 (−823).

Full enwik8:
```
ROUNDTRIP=OK COMPRESSED=21913905 DECOMPRESSOR=37304 SCORE=21951209 COMPRESS_SECONDS=343.83 DECOMPRESS_SECONDS=333.06 PEAK_RSS_MB=1756
```

| | Before (cross) | After | Δ |
|--|--:|--:|--:|
| COMPRESSED | 21918203 | 21913905 | **−4298** |
| DECOMPRESSOR | 37288 | 37304 | +16 |
| SCORE | 21955491 | **21951209** | **−4282** |

Gap to 20.5e6 ≈ 1.41 MB.

## Attempt 7 — StateMap on hashed Order-3 (KEEP)

`HashOrderModel::new_statemap` for o3 only (p+n cells, soft-reset). Distinct from rejected blanket u16 n0/n1 rescale.

Slice: **5650925** vs 5664646 (−13721).

Full enwik8:
```
ROUNDTRIP=OK COMPRESSED=21871189 DECOMPRESSOR=37544 SCORE=21908733 COMPRESS_SECONDS=344.23 DECOMPRESS_SECONDS=332.04 PEAK_RSS_MB=1788
```

| | Before (reset) | After | Δ |
|--|--:|--:|--:|
| COMPRESSED | 21913905 | 21871189 | **−42716** |
| DECOMPRESSOR | 37304 | 37544 | +240 |
| SCORE | 21951209 | **21908733** | **−42476** |

Gap to 20.5e6 ≈ 1.37 MB.

## Attempt 8 — StateMap on hashed Order-4 (KEEP)

Extend `new_statemap` to o4 (o3 already). Slice **5643727** vs 5650925 (−7198).

Full enwik8:
```
ROUNDTRIP=OK COMPRESSED=21849655 DECOMPRESSOR=37512 SCORE=21887167 COMPRESS_SECONDS=373.35 DECOMPRESS_SECONDS=345.21 PEAK_RSS_MB=1852
```

| | Before (SM-o3) | After | Δ |
|--|--:|--:|--:|
| COMPRESSED | 21871189 | 21849655 | **−21534** |
| DECOMPRESSOR | 37544 | 37512 | −32 |
| SCORE | 21908733 | **21887167** | **−21566** |

Gap to 20.5e6 ≈ 1.35 MB.

## Attempt 9 — StateMap on hashed Order-5 (REJECT)

Slice **5649546** vs o3+o4 base 5643727 (**+5819**). Reverted.

## Attempt 10 — StateMap on Indirect counters (REJECT)

Replace indirect Laplace n0/n1 with p+n cells (keep t3): slice **+22784**. Reverted.

## Attempt 11 — STRETCH drift / RSS trim

Not reached this round (time). Soft-reset / SM path left heap STRETCH as-is.

## FINAL REPORT (DIAG-R4)

### Verdict: **partial**

Clear SCORE wins vs DIAG-R3 baseline; **≤20.5e6 COMPRESSED not reached**.

### Best judge line

```
ROUNDTRIP=OK COMPRESSED=21849655 DECOMPRESSOR=37512 SCORE=21887167 COMPRESS_SECONDS=373.35 DECOMPRESS_SECONDS=345.21 PEAK_RSS_MB=1852
```

| | Baseline (R3) | Best (R4) | Δ |
|--|--:|--:|--:|
| COMPRESSED | 22039256 | **21849655** | **−189601** |
| DECOMPRESSOR | 36680 | 37512 | +832 |
| SCORE | 22075936 | **21887167** | **−188769** |
| bpB | 1.76314 | 1.74797 | |

Gap to 20.5e6: **≈1.35 MB** (was 1.54 MB).

### Commits (agent/diag-r4)

| Commit | Change | SCORE |
|--------|--------|------:|
| `3163073` | Order-2 count-dependent StateMap | 22045610 |
| `e405da9` | indirect order-3 last-byte (t3) | 21970140 |
| `896dd9d` | match×indirect cross MIX=16 | 21955491 |
| `589cfe1` | StateMap soft-reset n→208 | 21951209 |
| `15af354` | StateMap hashed Order-3 | 21908733 |
| `94c24eb` | StateMap hashed Order-4 | **21887167** |

### Acceptance

- Judge ROUNDTRIP=OK and SCORE < 22075936: **YES**
- ≤20.5e6 COMPRESSED: **NO**
- Adversarial: **10/10 PASS** on each kept commit

### Rejected

| Hyp | Evidence |
|-----|----------|
| Fixed dual-rate p_fast/p_slow replacing O2 Laplace | slice +910 KB |
| Match secondary hash o4 (loose / strict) | slice +80910 / +24 |
| Indirect t3 + StateMap cells combo | slice +8282 |
| StateMap Order-1 | slice −3 (noise) |
| StateMap Order-5 | slice +5819 |
| StateMap Indirect counters | slice +22784 |

### Next ideas (toward 20.5e6)

1. StateMap on Order-6/7 with smaller tables or only when mix weight justifies RSS.
2. Alt-salt order-6 secondary match table (only when primary chain misses).
3. Fix STRETCH drift (+2 KB COMPRESSED vs static LUT) if bit-identical reclaim possible.
4. Richer match×indirect (ind2/ind3 in cross) or APM cxt × ind nibble.
5. Bit-history state machine per dense context + dual-rate map (hybrid with Laplace).
6. RSS trim: pack match chain / shrink HASH_BITS before enwik9 (PEAK_RSS ~1.85 GB).

# DIAG-R5

Branch: `agent/diag-r5`  
Agent: DIAG-R5  
Start: 2026-11-24  
Baseline to beat: `ROUNDTRIP=OK COMPRESSED=21849655 DECOMPRESSOR=37512 SCORE=21887167`

## Plan

1. StateMap on hashed o6/o7 carefully (o3/o4 worked; o5 rejected; prove determinism).
2. Alt-salt match when primary misses.
3. STRETCH bit-identical reclaim (fix +2KB COMPRESSED drift vs static LUT).
4. Richer ind2/ind3 cross.
5. RSS trim if SCORE-neutral.

## Attempt 1 — StateMap on hashed Order-6 (REJECT)

`HashOrderModel::new_statemap(6, ORDER6_BITS)`; leave o5/o7 Laplace.

Slice **5645665** vs 5643727 (**+1938**). Reverted.

## Attempt 2 — StateMap on hashed Order-7 (REJECT)

`new_statemap(7, ORDER7_BITS)` only.

Slice **5644104** vs 5643727 (**+377**). Reverted.

## Attempt 3 — Alt-salt match on primary miss (REJECT)

Secondary `ht2` (HASH2_BITS=24) with ALT_SALT over same HASH_ORDER=6; lookup **only** when primary+chain miss.

Slice **5643727** (delta 0) — SCORE-neutral, +~64 MiB RSS. Reverted.

## Attempt 4 — Richer ind2 cross nibble (KEEP)

Expand cross StateMap: `mb × ind1 × ind2_hi_nibble × bit_pos × expected_bit` (`CROSS_SIZE` 32768→524288). Keep `MATCH_IND_CROSS_WEIGHT=16`.

Rejected variants (same attempt):
- XOR-fold ind2/ind3 into ind1 byte (same size): slice **+2474**
- ind2_nibble XOR ind3 (same expanded size): slice **-299** (worse than pure ind2)

Slice (pure ind2 nibble): **5636937** vs 5643727 (**−6790**).

Full enwik8:
```
ROUNDTRIP=OK COMPRESSED=21826291 DECOMPRESSOR=37560 SCORE=21863851 COMPRESS_SECONDS=362.49 DECOMPRESS_SECONDS=337.15 PEAK_RSS_MB=1853
```

| | Before | After | Δ |
|--|--:|--:|--:|
| COMPRESSED | 21849655 | 21826291 | **−23364** |
| DECOMPRESSOR | 37512 | 37560 | +48 |
| SCORE | 21887167 | **21863851** | **−23316** |

Gap to 20.5e6 ≈ 1.33 MB. Adversarial: **10/10 PASS**

## Attempt 5 — MATCH_IND_CROSS_WEIGHT retune (KEEP)

Slice sweep vs ind2-cross base 5636937:

| MIX | Slice | Δ |
|----:|------:|--:|
| 12 | 5638555 | +1618 |
| 16 | 5636937 | 0 |
| 20 | 5635672 | −1265 |
| **22** | **5635389** | **−1548** |
| 24 | 5635428 | −1509 |
| 26 | 5635544 | −1393 |
| 28 | 5635766 | −1171 |
| 32 | 5637021 | +84 |

Full enwik8 (MIX=22):
```
ROUNDTRIP=OK COMPRESSED=21819876 DECOMPRESSOR=37544 SCORE=21857420 COMPRESS_SECONDS=370.43 DECOMPRESS_SECONDS=358.17 PEAK_RSS_MB=1854
```

| | Before | After | Δ |
|--|--:|--:|--:|
| COMPRESSED | 21826291 | 21819876 | **−6415** |
| DECOMPRESSOR | 37560 | 37544 | −16 |
| SCORE | 21863851 | **21857420** | **−6431** |

Gap to 20.5e6 ≈ 1.32 MB. Adversarial: **10/10 PASS**.

## Attempt 6 — STRETCH static reclaim (REJECT / N/A)

Const-evaluated static LUT vs heap OnceLock: slice **5635389** (delta 0). Values already bit-identical; historical +2 KB COMPRESSED drift not reproducible. Keep heap (smaller DECOMPRESSOR).

## Attempt 7 — Cross + ind3 2-bit expand (REJECT)

`CROSS_SIZE` 524288→2097152 with ind3>>6. Slice **5635913** vs 5635389 (**+524**). Reverted.

## Attempt 8 — APM cxt × ind1 nibble (REJECT)

Fold ind1 high nibble into APM `last_byte` nibble. Slice **+41550**. Reverted.

## Attempt 9 — StateMap soft-reset n→1 (KEEP)

Retune soft-reset target after saturation (was 208). Slice sweep (vs MIX22 base 5635389):

| n | Δ |
|--:|--:|
| 224 | +588 |
| 208 | 0 |
| 192 | −492 |
| 144 | −2783 |
| 96 | −5636 |
| 48 | −9773 |
| 16 | −15016 |
| 4 | −17593 |
| **1** | **−18027** |
| 2 | −17953 |
| 0 | −17646 |

Full enwik8 (n=1):
```
ROUNDTRIP=OK COMPRESSED=21740866 DECOMPRESSOR=37544 SCORE=21778410 COMPRESS_SECONDS=362.03 DECOMPRESS_SECONDS=382.10 PEAK_RSS_MB=1853
```

| | Before | After | Δ |
|--|--:|--:|--:|
| COMPRESSED | 21819876 | 21740866 | **−79010** |
| DECOMPRESSOR | 37544 | 37544 | 0 |
| SCORE | 21857420 | **21778410** | **−79010** |

Gap to 20.5e6 ≈ 1.24 MB. Adversarial: **10/10 PASS**.

## FINAL REPORT (DIAG-R5)

### Verdict: **partial**

Clear SCORE wins vs DIAG-R4 baseline; **≤20.5e6 COMPRESSED not reached**.

### Best judge line

```
ROUNDTRIP=OK COMPRESSED=21740866 DECOMPRESSOR=37544 SCORE=21778410 COMPRESS_SECONDS=362.03 DECOMPRESS_SECONDS=382.10 PEAK_RSS_MB=1853
```

| | Baseline (R4) | Best (R5) | Δ |
|--|--:|--:|--:|
| COMPRESSED | 21849655 | **21740866** | **−108789** |
| DECOMPRESSOR | 37512 | 37544 | +32 |
| SCORE | 21887167 | **21778410** | **−108757** |
| bpB | 1.74797 | 1.73927 | |

Gap to 20.5e6: **≈1.24 MB** (was 1.35 MB).

### Commits (agent/diag-r5)

| Commit | Change | SCORE |
|--------|--------|------:|
| `ab9ee4e` | richer match×indirect cross ind2 nibble | 21863851 |
| `54e707f` | MATCH_IND_CROSS_WEIGHT=22 | 21857420 |
| `d0c7e04` | StateMap soft-reset n→1 | **21778410** |

### Acceptance

- Judge ROUNDTRIP=OK and SCORE < 21887167: **YES**
- ≤20.5e6 COMPRESSED: **NO**
- Adversarial: **10/10 PASS** on each kept commit

### Rejected

| Hyp | Evidence |
|-----|----------|
| StateMap hashed Order-6 | slice +1938 |
| StateMap hashed Order-7 | slice +377 |
| Alt-salt match on primary miss | slice Δ0, +RSS |
| Cross XOR-fold ind2/ind3 into ind1 | slice +2474 |
| Cross ind2^ind3 nibble | slice −299 (worse than pure ind2) |
| Cross + ind3 2-bit expand | slice +524 |
| STRETCH static reclaim | slice Δ0 (already identical) |
| APM cxt × ind1 nibble | slice +41550 |

### Next ideas (toward 20.5e6)

1. Soft-reset schedule (per-order or decay toward mid) vs fixed n→1.
2. Match chain depth / fingerprint on prev slots (alt-salt miss was neutral).
3. Word / o5 Laplace→hybrid dual-rate without full StateMap (o5 SM rejected).
4. RSS trim: pack match `ht_prev` / shrink HASH_BITS before enwik9 (PEAK_RSS ~1.85 GB).
5. Mixer weight retune after soft-reset=1 (ORDER3/5/6, SSE, APM).

# DIAG-R6

Branch: `agent/diag-r6`  
Agent: DIAG-R6  
Start: 2026-11-24  
Baseline to beat: `ROUNDTRIP=OK COMPRESSED=21740866 DECOMPRESSOR=37544 SCORE=21778410`

## Plan

1. Soft-reset schedule (per-order / decay / asymptotic) vs fixed n→1.
2. Mixer weight retune after new schedule (ORDER3/5/6, SSE, APM, CROSS).
3. Word / o5 Laplace→hybrid dual-rate (no full StateMap on o5).
4. RSS trim only if SCORE-neutral+.

Slice baseline (first 20e6 enwik8): **4510861**.

## Attempt 1 — Soft-reset schedule variants (REJECT modes 1–3; KEEP asymptotic)

| Mode | Description | Slice | Δ |
|-----:|-------------|------:|--:|
| 0 | fixed n→1 (R5) | 4510861 | 0 |
| 1 | surprise-scaled reset | 4515432 | +4571 |
| 2 | mispred-gated | 4512093 | +1232 |
| 3 | halve →127 | 4520958 | +10097 |
| 4 | asym n+=1-(n>>6) eq≈64 | 4502012 | −8849 |
| 5 | asym >>5 eq≈32 | 4487900 | −22961 |
| 6 | asym >>4 eq≈16 | 4477478 | −33383 |
| **7** | **asym >>3 eq≈8** | **4476865** | **−33996** |
| 8 | asym >>2 eq≈4 | 4488669 | −22192 |
| 9 | asym >>1 eq≈2 | 4509359 | −1502 |

Per-order fixed resets (o2/o3/o4/cross): best was cross=16 (−907); noise / reject vs asymptotic.

Full enwik8 (asym >>3):
```
ROUNDTRIP=OK COMPRESSED=21546316 DECOMPRESSOR=37560 SCORE=21583876 COMPRESS_SECONDS=346.95 DECOMPRESS_SECONDS=339.65 PEAK_RSS_MB=1853
```

| | Before | After | Δ |
|--|--:|--:|--:|
| COMPRESSED | 21740866 | **21546316** | **−194550** |
| DECOMPRESSOR | 37544 | 37560 | +16 |
| SCORE | 21778410 | **21583876** | **−194534** |

Gap to 20.5e6 ≈ 1.05 MB. Adversarial: **10/10 PASS**.


---

## PRE-V2 word dict trial (agent PRE-V2-WORD)

Start: 2026-11-25
Branch: `agent/pre-v2`
Baseline: `ROUNDTRIP=OK COMPRESSED=21546316 DECOMPRESSOR=37560 SCORE=21583876`

### Implementation
- Split preprocessor into `preprocessor/{mod,stages,capitals,markup,words}.rs`
- Added `tools/build_word_dict.py` (two-pass, memory-capped, NL-only)
- Stage flags: `PRE_STAGE_WORDS` (default false), `WORD_DICT_N`

### Phase A — dict build from enwik9
(in progress; see judge_work/build_word_dict.log)


### Phase A result
- Blob sizes: 4096=47482B, 16384=201320B, 49152=647120B
- Selected N=4096 (`the` top; savings-ranked; wiki-boilerplate blacklisted)
- Word-boundary matching + 1-byte tokens for top 90

### Phase B — words ON (cycle 0 = first judge)
```
ROUNDTRIP=OK COMPRESSED=21279587 DECOMPRESSOR=86160 SCORE=21365747 COMPRESS_SECONDS=307.98 DECOMPRESS_SECONDS=290.25 PEAK_RSS_MB=1834
```
- Δ vs baseline SCORE **21583876**: **−218129** (KEEP immediately)
- COMPRESSED −266729; DECOMPRESSOR +48600 (blob ~47KB embedded)
- Adversarial: **10/10 PASS**
- Cycles used: **0/15** (no tune needed)

## PRE-V2 word dict trial
- N tried: 4096
- Cycles used: 0/15
- Best SCORE with dict: `ROUNDTRIP=OK COMPRESSED=21279587 DECOMPRESSOR=86160 SCORE=21365747`
- Baseline: `ROUNDTRIP=OK COMPRESSED=21546316 DECOMPRESSOR=37560 SCORE=21583876`
- Verdict: KEEP
- Blob N=4096; DECOMPRESSOR delta = +48600 (37560 → 86160)
- Next: TEMPLATE-MINER (orch will spawn)


## B1-HASH v4 KEEP (ORDER3/4=27 + checksums)

```
ROUNDTRIP=OK COMPRESSED=20824740 DECOMPRESSOR=86880 SCORE=20911620
COMPRESS_SECONDS=400.83 DECOMPRESS_SECONDS=400.88 PEAK_RSS_MB=3562
```
Δ vs 21365747: **−454127**. Slice 20MB −69771; ratio OK. Adv **10/10**.
enwik9 estimate: time ≳10× (~4000s+), RAM ~3.5–6GB (flag larger tables for manual check).
