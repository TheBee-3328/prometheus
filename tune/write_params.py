#!/usr/bin/env python3
"""Rewrite the BEGIN/END TUNABLE block in params.rs from JSON/CLI args."""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
PARAMS = ROOT / "crates/prometheus-core/src/params.rs"

DEFAULTS = {
    "ORDER2_RESCALE": 250,
    "BUF_BITS": 24,
    "HASH_BITS": 23,
    "HASH_ORDER": 4,
    "MIN_MIX_LEN": 5,
    "MATCH_CONF": [14000, 20000, 26000, 30000, 32000],
    "MIX_WEIGHT": [100, 160, 200, 230, 245],
    "STRUCT_MIX_WEIGHT": 14,
}

BLOCK_RE = re.compile(
    r"// === BEGIN TUNABLE ===.*?// === END TUNABLE ===",
    re.DOTALL,
)


def render(p: dict) -> str:
    conf = ", ".join(str(int(x)) for x in p["MATCH_CONF"])
    mix = ", ".join(str(int(x)) for x in p["MIX_WEIGHT"])
    return f"""// === BEGIN TUNABLE ===
/// Order-2 counter rescale when n0+n1 reaches this (adaptation rate).
pub const ORDER2_RESCALE: u16 = {int(p['ORDER2_RESCALE'])};

/// Match history buffer size = 1 << BUF_BITS.
pub const BUF_BITS: usize = {int(p['BUF_BITS'])};
/// Match hash table size = 1 << HASH_BITS.
pub const HASH_BITS: usize = {int(p['HASH_BITS'])};
/// Bytes hashed for match lookup.
pub const HASH_ORDER: usize = {int(p['HASH_ORDER'])};
/// Minimum match length before ensemble uses match probabilities.
pub const MIN_MIX_LEN: u32 = {int(p['MIN_MIX_LEN'])};

/// Match-model confidence (offset from 32768) by match-length bucket:
/// [5..=6, 7..=10, 11..=20, 21..=40, 41+].
pub const MATCH_CONF: [u32; 5] = [{conf}];

/// Ensemble mix weight (0..=255) for match vs order-2, same buckets.
pub const MIX_WEIGHT: [u32; 5] = [{mix}];

/// When match is inactive: blend weight (0..=255) of XML/wiki structure into the mix.
pub const STRUCT_MIX_WEIGHT: u32 = {int(p['STRUCT_MIX_WEIGHT'])};
// === END TUNABLE ==="""


def write_params(p: dict) -> None:
    text = PARAMS.read_text()
    block = render(p)
    new, n = BLOCK_RE.subn(block, text, count=1)
    if n != 1:
        sys.exit(f"write_params: expected 1 TUNABLE block, found {n}")
    PARAMS.write_text(new)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", help="params as JSON object")
    ap.add_argument("--file", help="path to JSON params file")
    ap.add_argument("--defaults", action="store_true")
    args = ap.parse_args()
    if args.defaults:
        p = dict(DEFAULTS)
    elif args.file:
        p = json.loads(pathlib.Path(args.file).read_text())
    elif args.json:
        p = json.loads(args.json)
    else:
        sys.exit("need --json, --file, or --defaults")
    for k, v in DEFAULTS.items():
        p.setdefault(k, v)
    write_params(p)
    print(json.dumps(p, separators=(",", ":")))


if __name__ == "__main__":
    main()
