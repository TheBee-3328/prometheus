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
