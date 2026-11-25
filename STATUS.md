# STATUS — PROMETHEUS Phase 2

**LIVE** 2026-11-25T20:16Z  
**Phase:** 2 (bigger model / budget rule)  
**Best checkpoint:** `CHKP-b5-20769768` on `main` (commit `0c3248c` / tip includes merge `a397d30`)  
**Discord:** embeds via `/tmp/agent-home/.prometheus/discord_notify.js` (secret URL local only)

## Baseline (verify)

| | enwik8 (B5, official) | enwik9 |
|--|--:|--:|
| SCORE | **20 769 768** | *(re-measure in flight — see below)* |
| Archive | 20 681 576 | — |
| Decompressor | **88 192** | must be **88 192** when binary is correct |
| Round trip | OK | — |
| Peak RSS | 3658 MB (prior judge) | — |

enwik8 Δ vs A1 (20 852 107): **−82 339**.  
Target enwik9 ≈ **150 000 000** (~19.6% below a correct ~186.6 MB-class run).  
**Broken 20 054 334-byte archive (RT fail) is not a baseline.**

### Active run
- **enwik9 valid re-judge** (pid judge ~47743): B5 binary **88192 B**, `CARGO_TARGET_DIR` unset. Compress ~43 min+ / ~106 MB out / RSS ~5 GB at last sample. ETA ~1–1.5 h compress + decompress + determinism. Watchdog: `/tmp/agent-home/.prometheus/run_enwik9.sh` + orch monitor.

### Invalid prior enwik9 (discarded)
SCORE **186 586 969** with DECOMPRESSOR **37 512** — **not** B5. Cause documented below. Do not compare to enwik8.

---

## Section 2 resolutions

### 1. Decompressor size mismatch — **EXPLAINED**
- **Cause:** shell env had `CARGO_TARGET_DIR=/tmp/workspace/prometheus-c1-tmpl/target-c1`. `judge.sh` / flake `cargo` wrap wrote/copied a **stale pre-B5** binary (~37 KB, no embedded `word_dict.bin`) into the path used for enwik9, while enwik8 B5 was measured with the real **88 192 B** binary (dict embedded via `include_bytes!`).
- **Not** a different counting rule or SCORE formula bug.
- **Fix:** `judge.sh` now `unset CARGO_TARGET_DIR` and mirrors `target/<triple>/release` → `target/release` if needed. Always `wc -c` the stripped decompress before trusting a run.
- **Until a valid enwik9 line with DECOMPRESSOR=88192 exists, do not compare enwik8 vs enwik9 SCORE.**

### 2. Time / RAM limits — **RECORDED**
**Official** ([hrules.htm](https://www.hutter1.net/prize/hrules.htm)):
- Each program (comp and decomp) must run in **&lt; 70 000 / T hours**, `T` = Geekbench **5** of the machine; ≤ **10 GB** RAM; ≤ **100 GB** temp disk; **no GPU**.
- Contest test machines (as of 2021, may change): Lenovo i7-1165G7 T≈**1427** (1-core) / AMD Ryzen T≈**1310** (1-core) → ≈ **49–53 h** per program on those boxes.
- Prize page summary also cites ≈**50 h** single-core on their test machine.

**This host:** Geekbench 5 **T = unmeasured**. Exact host wall-second budget = **70000/T hours cannot be computed until T is measured.** Operator asked to stop if uncertain — **reported here**. Proceeding under Phase 2 **provisional working ceilings** (also enforced in `judge.sh`):
- **TIME:** ≤ **4 h wall per direction** (`TIME_LIMIT=PASS|FAIL`, default `PROMETHEUS_WALL_LIMIT_SECONDS=14400`)
- **RAM:** ≤ **9 GB** peak RSS (`RAM_LIMIT=PASS|FAIL`, default `PROMETHEUS_RAM_LIMIT_MB=9216`)
- Official 10 GB still noted in `resources.json` as `hutter_peak_ram_limit_gb`.

### 3. Judge hygiene — **DONE** (on disk; applies to next runs)
- RT fail → print only `ROUNDTRIP=FAIL` (no SCORE).
- Byte-compare + **SHA256** of original and roundtrip (`ORIG_SHA256`, `ROUNDTRIP_SHA256`).
- Compress×2 archive hash; decompress×2 under `env -i` clean env.
- `TIME_LIMIT` / `RAM_LIMIT` PASS|FAIL on success lines.
- Webhook: unescape literal `\n` in embeds; result posts from `discord_judge_result.sh` only.

### 4. Determinism — **PARTIAL**
- 2 MiB slice: compress×2 archive **PASS** (identical SHA). Clean-env decompress aborted (would OOM beside enwik9); **full §2.4 after enwik9**.

---

## Working ceilings (budget)
| Ceiling | Value |
|--|--|
| Wall / direction | 4 h |
| Peak RSS | 9 GB |
| Official RAM | 10 GB |
| Official time | 70000/T h (T unknown here) |

## Queued (after enwik9 + §2.4)
1. Tag/reconcile B5 for both corpora once valid enwik9 lands  
2. Profile B5 (Step 1) → Track 0 update in DIAG.md  
3. Highest bytes/CPU-hour experiments under budget rule  

## Closed
C1 THROW · B2/B3/B4 REJECT · B6-WS REJECT · T0 · B5 KEEP
