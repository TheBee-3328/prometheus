# STATUS — PROMETHEUS Phase 2

**LIVE** 2026-11-26T01:00Z  
**Best checkpoint:** `CHKP-b5-20769768`  
**License:** GPLv3-or-later

## Locked results
| | enwik8 | enwik9 |
|--|--:|--:|
| SCORE | **20 769 768** | **173 961 591** |
| Archive | 20 681 576 | 173 873 383 |
| Decompressor | 88 192 | **88 208** (same B5 build; strip Δ) |
| Peak RSS | 3658 MB | **5973 MB** |
| Wall c+d | ~834 s | **7654 s** (~2.13 h) |
| Limits | — | TIME **PASS** · RAM **PASS** (9 GB working) |

Target ~150 MB SCORE: still **~24 MB** above. Invalid prior 186 586 969 discarded.

## Active
- **[P2-SPEED](6ee6c78c-db59-4d68-9038-c1e8cf955b8b)** `agent/p2-speed` bit-identical CPU opts

## Next
1. Land speed results → spawn `PHASE2_MATCH_HASH_BUDGET`  
2. Preprocess / CM scale per ablation (SSE/APM/o3/o5/word)  
3. Neural blocked until speed/CM ratios known
