# STATUS

**Current phase:** 0 (Judge harness)  
**Current best SCORE:** none (no measured baseline yet)  
**Best measured on:** —  
**Agent runs used:** 0 / 40  
**Wall clock budget:** 8 hours from Phase 0 start  

## Running tasks

| Task ID | Agent | Status |
|---|---|---|
| P0-JUDGE | JUDGE | spawning |

## Known risks

- `enwik9` is 1 GB on disk; do not commit it.
- Official Hutter S differs from our SCORE (we omit compressor size / 2× decomp); do not confuse them in claims.
- Bit-identical compressor/decompressor model path is mandatory; FP or HashMap order will fail determinism.

## Next three tasks

1. Complete Phase 0: JUDGE harness + baseline numbers in LOG.md.
2. Phase 1: cargo workspace skeleton, traits, arithmetic coder, order-2 model; pass round-trip on enwik8.
3. Phase 2 kickoff: parallel builders (orders 1–6, match, word, mixer+APM, preprocessor).
