# P2-IND2 — KEEP (2026-11-26)

**Lane:** `agent/p2-ind2` @ tip SCORE **20415163** (O6 KEEP)  
**Hypothesis family:** ORDER0–CROSS residual + IND hit-gate (with IND bits/mix stack)  
**Tip slice25:** **5610357** (WALL≈105.4s, RSS≈8380 MB)  
**Winner:** `ORDER0_MIX=40` + `MATCH_IND_CROSS=8` + `INDIRECT_BITS=28` + `INDIRECT_MIX=40` + IND hit-gate (`pi != 32768`)

## Trials (slice25 vs tip 5610357) — selected

| ID | Change | Bytes | Δ | Gate |
|--|--|--:|--:|--|
| tip | baseline | 5610357 | 0 | — |
| indb28 | BITS=28 | 5608288 | −2069 | FAIL |
| indmix48 | MIX=48 | 5622144 | +11787 | FAIL |
| o0_48_c8 | O0=48 CROSS=8 | 5602531 | −7826 | FAIL |
| o0_40_c8_b28 | +BITS=28 | 5599894 | −10463 | FAIL |
| o0_40_c8_b29 | +BITS=29 | 5599648 | −10709 | FAIL |
| hit_tip | hit-gate only | 5605910 | −4447 | FAIL |
| hit_w40c8_b28 | hit+O0/CROSS/b28 MIX=32 | 5597284 | −13073 | FAIL |
| hit_w40c8_b28_i36 | +IND_MIX=36 | 5595175 | −15182 | **PASS** |
| **hit_w40c8_b28_i40** | **+IND_MIX=40** | **5594393** | **−15964** | **PASS** |

Full log: `tune/p2_ind2_trials.csv`. IND MIX↑ alone toxic; hit-gate unlocks MIX=40. ORDER0_MODE class/nibble enrich rejected (+11–19KB).

## enwik8 judge

```
ROUNDTRIP=OK COMPRESSED=20265992 DECOMPRESSOR=87488 SCORE=20353480
COMPRESS_SECONDS=398.16 DECOMPRESS_SECONDS=384.92 PEAK_RSS_MB=8968
CPU_TOTAL_SECONDS=780.02 WALL_TOTAL_SECONDS=783.08 RAM_TIME_GB_H=1.9048
TIME_LIMIT=PASS RAM_LIMIT=PASS
ARCHIVE_SHA256=68db58c24449d57bfca7411c5021ac4c1c18030f4297a8b94b44e55d7556438b
```

Δ SCORE vs tip 20415163: **−61683**

## Adversarial

`nix develop -c ./tests/adversarial_roundtrip.sh` → **PASS=10 FAIL=0** (exclusive after judge).

## Verdict

**KEEP** — SCORE < tip, ROUNDTRIP=OK, TIME/RAM PASS, adv 10/10.  
Do **not** merge without orch.

## Notes

- Family: ORDER0–CROSS residual (O0 64→40, CROSS 16→8) + IND hit-gate + BITS 26→28 + MIX 32→40.
- Pure IND bit-grow (O6 pattern) capped ~−2KB; MIX↑ without hit-gate regresses hard.
- No SSE/MATCH2/O6 re-sweeps. No neural. No merge.

## RAM

NO_CAP · peak RSS 8968 MB enwik8 · **UNVERIFIED FOR 10 GB**

**Adv:** **10/10 PASS** (exclusive adversarial_roundtrip.sh)
