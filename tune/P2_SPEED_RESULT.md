# P2-SPEED RESULT — bit-identical microopts (KEEP)

**Verdict:** KEEP  
**Bit-identical tip archive:** YES  
- size: 5680631  
- sha256: `d400abbca2506ad3b5b5ae8353e71c7c1f386106e567340f3ac90f874adc60da`  
- verified: opt×2 + paired tip/opt archives all match tip

**RAM:** NO_CAP · peak RSS ~3.38 GB · **UNVERIFIED FOR 10 GB**

## Paired GNU-time retime (`tune/slice25.bin`, idle-ish host)

| run | WALL_s | CPU_USER_s | CPU_SYS_s | RSS_KB |
|--|--:|--:|--:|--:|
| tip A | 126.33 | 124.12 | 1.77 | 3548404 |
| opt A | 94.64 | 92.59 | 1.72 | 3548584 |
| opt B | 95.18 | 93.06 | 1.80 | 3548372 |
| tip B | 123.47 | 121.30 | 1.74 | 3548588 |

**Means (A+B):** tip WALL **124.90s** / RSS **~3548 MB** → opt WALL **94.91s** / RSS **~3548 MB**  
**Δ WALL:** −29.99 s (−24.0%) · **Δ CPU_USER:** −29.89 s

## CPU-hours saved (estimate)

- slice25: ~0.0083 CPU-h  
- enwik8 (~×4): ~0.033 CPU-h wall/CPU  
- enwik9 projected (~×10–12 of enwik8): ~0.33–0.40 CPU-h  

Speed is **budget**, not SCORE. No prediction math / mix / table-size changes.

## What changed (hot path only)

- Pack n0/n1 into `u16` (SSE, order1/2/indirect/word) — one cache touch per slot  
- Cache predict→update index (avoid rehash)  
- APM power-of-two mask; drop redundant bounds check  
- Match hash unroll (HASH_ORDER==6) + ht prefetch  
- StateMap `clamp` instead of manual branches  

enwik8 full retime: not run in this pass (slice signal clear).
