# Benchmark data (sliced for GitHub)

GitHub blocks files ≥ 100 MB, so we do **not** commit `enwik8` / `enwik9` whole.
Parts are **exactly 50 000 000 bytes** each (≈47.7 MiB).

| File | Parts | Layout |
|---|---|---|
| `enwik8` (1e8 bytes) | `enwik8.parts/enwik8.part_00` … `_01` (2) | concat in order |
| `enwik9` (1e9 bytes) | `enwik9.parts/enwik9.part_00` … `_19` (20) | concat in order |

## Reconstruct

```bash
chmod +x scripts/reconstruct_data.sh
./scripts/reconstruct_data.sh both    # or: enwik8 | enwik9
```

Verifies byte counts and `SHA256SUMS` in each parts directory.

## Alternate source

Official dump: https://prize.hutter1.net / enwik9.zip — then `head -c 100000000 enwik9 > enwik8`.
