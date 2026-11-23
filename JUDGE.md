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

## Output (exactly one summary line)

```
ROUNDTRIP=OK|FAIL COMPRESSED=<bytes> DECOMPRESSOR=<bytes> SCORE=<sum> COMPRESS_SECONDS=<n> DECOMPRESS_SECONDS=<n> PEAK_RSS_MB=<n>
```

| Field | Meaning |
|---|---|
| `ROUNDTRIP` | `OK` iff decompress succeeds and `cmp` matches the original; also requires compress×2 and decompress×2 bit-identical |
| `COMPRESSED` | Byte size of the compressed archive |
| `DECOMPRESSOR` | Stripped `prometheus-decompress` executable **plus** every auxiliary file the decompressor reads at runtime |
| `SCORE` | `COMPRESSED + DECOMPRESSOR` |
| `COMPRESS_SECONDS` | Wall seconds (GNU `time` `%e`) for one compress |
| `DECOMPRESS_SECONDS` | Wall seconds for one decompress |
| `PEAK_RSS_MB` | Max of compress/decompress peak RSS (`time` `%M` KB → MB, ceil) |

Exit status `0` only when `ROUNDTRIP=OK`.

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
