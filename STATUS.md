# STATUS

**Current phase:** 1 (workspace + arithmetic coder + order-2)  
**Current best SCORE:** **100299876** (enwik8, store baseline)  
**Best measured on:** enwik8 via `nix develop -c ./judge.sh enwik8 .`  
**Baseline line:** `ROUNDTRIP=OK COMPRESSED=100000012 DECOMPRESSOR=299864 SCORE=100299876 COMPRESS_SECONDS=0.14 DECOMPRESS_SECONDS=0.13 PEAK_RSS_MB=98`  
**Agent runs used:** 1 / 40 (P0-JUDGE done)  
**Wall clock budget:** 8 hours from Phase 0 start  

## Running tasks

| Task ID | Agent | Status |
|---|---|---|
| P1-BUILD | BUILDER | spawning |

## Known risks

- **NixOS:** all builds/judge runs via `nix develop -c …` (`flake.nix`). Host `~/.cargo` is not authoritative. Paths with spaces need `NIX_LDFLAGS` cleared (handled in flake/judge).
- `enwik9` is 1 GB on disk; do not commit it.
- Official Hutter S differs from our SCORE (we omit compressor size / 2× decomp); do not confuse them in claims.
- Bit-identical compressor/decompressor model path is mandatory; FP or HashMap order will fail determinism.
- Store baseline SCORE ≈ input size; Phase 1 must beat this on enwik8.

## Next three tasks

1. Phase 1: shared model crate, arithmetic coder, order-2 model; beat store SCORE on enwik8.
2. Phase 2: parallel builders (orders 1–6, match, word, mixer+APM, preprocessor).
3. Phase 3: reviewer + tuner.
