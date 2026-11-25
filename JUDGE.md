# JUDGE — evaluation harness

Phase 0 harness for PROMETHEUS. Measures the internal SCORE from `SPEC.md` without changing that contract.

## Nix

This project is Nix-only for authoritative builds.

```bash
nix develop -c ./judge_selftest.sh
nix develop -c ./judge.sh enwik8 .
nix develop -c ./judge.sh enwik9 .
```

**`judge.sh` / `judge_selftest.sh` nix policy:** Prefer running already inside `nix develop` (or via `nix develop -c …`). If `cargo` / `rustc` / `strip` / `cmp` / GNU `time` are missing from `PATH`, the script **re-execs** itself with `nix develop <repo> -c "$0" …` (guarded by `PROMETHEUS_NIX_REEXEC=1` to avoid loops). Do not use host `~/.cargo` as authoritative.

**Path spaces:** If the repo lives under a directory name with spaces, `mkShell` may set `NIX_LDFLAGS=-rpath $PWD/outputs/out/lib` which breaks `ld`. The flake `shellHook` and judge scripts clear `NIX_LDFLAGS` when it contains spaces.

Flake tools: `rustc`, `cargo`, `rustfmt`, `clippy`, `binutils` (strip), `coreutils`, `diffutils` (cmp), `time`, `gawk`, `gnused`, `gnugrep`, `findutils`, `bash`, `git`.

## CLI under test

Built from the Cargo workspace at repo root:

- `prometheus-compress <input> <output>`
- `prometheus-decompress <input> <output>`

Release profile (`Cargo.toml`): `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`. Judge also runs `strip` on the release binaries before measuring size.

## Usage

```bash
./judge.sh [enwik8|enwik9] [workspace=.]
```

- Dataset file must exist at repo root (`enwik8` / `enwik9`; gitignored).
- `workspace` is the Cargo workspace root (default `.`).
- Work dir: `judge_work/` (gitignored).

## Output

**Success** — one summary line (includes SCORE):

```
ROUNDTRIP=OK COMPRESSED=<bytes> DECOMPRESSOR=<bytes> SCORE=<sum> COMPRESS_SECONDS=<n> DECOMPRESS_SECONDS=<n> PEAK_RSS_MB=<n> CPU_* WALL_TOTAL_SECONDS=<n> RAM_TIME_GB_H=<n> TIME_LIMIT=PASS|FAIL RAM_LIMIT=PASS|FAIL ORIG_SHA256=<hex> ROUNDTRIP_SHA256=<hex> ARCHIVE_SHA256=<hex>
```

**Failure** — exactly:

```
ROUNDTRIP=FAIL
```

(no `SCORE=` field). Exit status non-zero.

Also writes `judge_work/resources.json` and `judge_work/<dataset>.hashes.txt`.

| Field | Meaning |
|---|---|
| `ROUNDTRIP` | `OK` iff decompress succeeds, `cmp` + SHA256 match original; compress×2 archive hash match; decompress×2 under clean `env -i` |
| `COMPRESSED` / `DECOMPRESSOR` / `SCORE` | Only on success. `SCORE = COMPRESSED + DECOMPRESSOR` (+ aux) |
| `COMPRESS_SECONDS` / `DECOMPRESS_SECONDS` | Wall seconds (GNU `%e`) |
| `PEAK_RSS_MB` | Max peak RSS; official Hutter ≤ 10240 MB |
| `CPU_*` | `%U+%S` single-core seconds |
| `TIME_LIMIT` | Working ceiling: wall ≤ 4 h/direction (`PROMETHEUS_WALL_LIMIT_SECONDS`, default 14400) |
| `RAM_LIMIT` | Working ceiling: peak ≤ 9 GB (`PROMETHEUS_RAM_LIMIT_MB`, default 9216) |
| `ORIG_SHA256` / `ROUNDTRIP_SHA256` / `ARCHIVE_SHA256` | Hashes for audit |

Exit `0` only when `ROUNDTRIP=OK` **and** both working ceilings PASS.

**Env:** judge always `unset CARGO_TARGET_DIR` so a poisoned worktree target cannot supply a stale binary.

## DECOMPRESSOR byte accounting

1. Size of stripped `target/release/prometheus-decompress`.
2. Plus each path listed in optional `judge_aux.txt` at the workspace root (one relative or absolute path per line; `#` comments allowed). Those files are weights/dictionaries/etc. that decompress opens at runtime.
3. Phase 0 store baseline has **no** aux files (`DECOMPRESSOR` = exe only).

Compressor size is **not** part of SCORE (dev convenience; official Hutter `S` differs — see `SPEC.md`).

## Hard rules checked

- Decompressor must reconstruct from the **archive alone** (never opens the original dataset).
- Corrupt archive must not silently round-trip (`judge_selftest.sh`).
- Determinism: two compresses of the same input → identical archives; two decompresses of the same archive → identical outputs.

## Phase 0 baseline

Store / copy-through format `PRM0`:

| Offset | Field |
|---|---|
| 0..3 | magic `PRM0` |
| 4..11 | `u64` LE uncompressed length |
| 12.. | raw payload |

No model. Phase 1 replaces the payload encoding; CLI and judge stay fixed.

## Self-test

```bash
nix develop -c ./judge_selftest.sh
```

Expects `SELFTEST PASS`. Verifies happy-path round-trip, that a corrupted archive fails, and compress/decompress determinism on a tiny fixture.
