# STATUS

**Current phase:** 2 (parallel context/mixer/preprocessor builders)  
**Current best SCORE:** **38254897** (enwik8, order-2)  
**Best measured on:** enwik8  
**Best line:** `ROUNDTRIP=OK COMPRESSED=37955033 DECOMPRESSOR=299864 SCORE=38254897 COMPRESS_SECONDS=12.11 DECOMPRESS_SECONDS=13.33 PEAK_RSS_MB=113`  
**Ratio vs raw:** ~2.62× (compressed alone); SCORE/1e8 ≈ 0.38  
**Agent runs used:** 2 / 40  
**Wall clock budget:** 8 hours from Phase 0 start  

## Running tasks

| Task ID | Agent | Status |
|---|---|---|
| P2-ORDERS | BUILDER | spawning |
| P2-MATCH | BUILDER | spawning |
| P2-WORD | BUILDER | spawning |
| P2-MIXER | BUILDER | spawning |
| P2-PRE | BUILDER | spawning |

## Known risks

- **NixOS:** `nix develop -c …` only.
- Parallel Phase 2 merge conflicts likely on `model.rs` / mains — merge one-at-a-time with rebase.
- Decompressor still ~300KB; size optimization later.
- Do not claim enwik9 results yet.

## Next three tasks

1. Merge each Phase 2 builder that improves SCORE.
2. Phase 3 reviewer after merge batch.
3. Phase 3 tuner on enwik8.
