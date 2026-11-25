# STATUS

**Current phase:** LIVE — chase ~10–12 MB enwik8 / ~95 MB enwik9  
**Poll cadence:** **2 minutes**  
**enwik9:** NEVER auto-start (manual only)

## Locked / current best enwik8 (on main)

```
ROUNDTRIP=OK COMPRESSED=20765227 DECOMPRESSOR=86880 SCORE=20852107
```
**bpB:** ≈ **1.661** · **Gap to ~11e6 mid-target:** ≈ **9.7 MB** · Gap to 20.5e6 soft: ≈ 0.35 MB  
**Mix:** ORDER0=64 CROSS=16 IND=32 ORDER5=72 · ORDER3/4=27 (B1)

## Ambition

- Operator target: **~10–12 MB** SCORE on enwik8 ⇒ roughly **~95 MB** enwik9.
- Mixer-only retunes cannot close this; need preprocessor (templates/numeric) + stronger models/contexts.

## enwik9 manual, may be stale

SCORE **186586969** (old binary)

## In flight

- **C1-TMPL:** structure gate PASS (ABCD cost share ≈18.6%; templates ≈15.4%). Mining next.
- **B2-MIXER:** 25MB SSE/APM/STRUCT sweeps (small expected Δ). Cap burn; one full judge for best combo only.
- Queued: A4 numeric, B3 contexts, larger dicts (manual enwik9)

## Done

- A3 word dict KEEP · B1 hash grow KEEP · A1 mix retune KEEP
