#!/usr/bin/env python3
"""Rewrite the full BEGIN/END TUNABLE block in params.rs from JSON."""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
PARAMS = ROOT / "crates/prometheus-core/src/params.rs"

BLOCK_RE = re.compile(
    r"// === BEGIN TUNABLE ===.*?// === END TUNABLE ===",
    re.DOTALL,
)

# Current main / PRE-V2 baseline
DEFAULTS = {
    "ORDER2_RESCALE": 250,
    "BUF_BITS": 25,
    "HASH_BITS": 26,
    "HASH_ORDER": 6,
    "MIN_MIX_LEN": 6,
    "MATCH_CONF": [32700, 32700, 32700, 32700, 32700],
    "MIX_WEIGHT": [120, 160, 190, 230, 245],
    "STRUCT_MIX_WEIGHT": 24,
    "ORDER1_MIX_WEIGHT": 8,
    "ORDER3_BITS": 24,
    "ORDER3_MIX_WEIGHT": 160,
    "ORDER4_BITS": 25,
    "ORDER4_MIX_WEIGHT": 48,
    "ORDER5_BITS": 24,
    "ORDER5_MIX_WEIGHT": 88,
    "ORDER6_BITS": 24,
    "ORDER6_MIX_WEIGHT": 32,
    "ORDER7_BITS": 23,
    "ORDER7_MIX_WEIGHT": 16,
    "SSE_MIX_WEIGHT": 224,
    "WORD_BITS": 23,
    "WORD_MIX_WEIGHT": 64,
    "SSE2_MIX_WEIGHT": 192,
    "ORDER0_MIX_WEIGHT": 80,
    "INDIRECT_BITS": 24,
    "INDIRECT_MIX_WEIGHT": 40,
    "MATCH_IND_CROSS_WEIGHT": 22,
    "APM_MIX_WEIGHT": 224,
    "APM_RATE_SHIFT": 7,
}


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

/// Blend weight (0..=255) of order-1 into order-2 before match/struct mix.
pub const ORDER1_MIX_WEIGHT: u32 = {int(p['ORDER1_MIX_WEIGHT'])};

/// Hashed order-3 table bits (size = 1 << this).
pub const ORDER3_BITS: usize = {int(p['ORDER3_BITS'])};
/// Blend weight (0..=255) of hashed order-3 into the CM mix.
pub const ORDER3_MIX_WEIGHT: u32 = {int(p['ORDER3_MIX_WEIGHT'])};

/// Hashed order-4 table bits.
pub const ORDER4_BITS: usize = {int(p['ORDER4_BITS'])};
/// Blend weight (0..=255) of hashed order-4 into the CM mix.
pub const ORDER4_MIX_WEIGHT: u32 = {int(p['ORDER4_MIX_WEIGHT'])};

/// Hashed order-5 table bits.
pub const ORDER5_BITS: usize = {int(p['ORDER5_BITS'])};
/// Blend weight (0..=255) of hashed order-5 into the CM mix.
pub const ORDER5_MIX_WEIGHT: u32 = {int(p['ORDER5_MIX_WEIGHT'])};

/// Hashed order-6 table bits.
pub const ORDER6_BITS: usize = {int(p['ORDER6_BITS'])};
/// Blend weight (0..=255) of hashed order-6 into the CM mix.
pub const ORDER6_MIX_WEIGHT: u32 = {int(p['ORDER6_MIX_WEIGHT'])};

/// Hashed order-7 table bits.
pub const ORDER7_BITS: usize = {int(p['ORDER7_BITS'])};
/// Blend weight (0..=255) of hashed order-7 into the CM mix.
pub const ORDER7_MIX_WEIGHT: u32 = {int(p['ORDER7_MIX_WEIGHT'])};

/// SSE blend weight (0..=255) of secondary estimate into final p.
pub const SSE_MIX_WEIGHT: u32 = {int(p['SSE_MIX_WEIGHT'])};

/// Word-model hash table bits.
pub const WORD_BITS: usize = {int(p['WORD_BITS'])};
/// Blend weight (0..=255) of word model into CM mix (before match/SSE).
pub const WORD_MIX_WEIGHT: u32 = {int(p['WORD_MIX_WEIGHT'])};

/// Second SSE (byte-class context) blend weight.
pub const SSE2_MIX_WEIGHT: u32 = {int(p['SSE2_MIX_WEIGHT'])};

/// Blend weight (0..=255) of order-0 into CM mix.
pub const ORDER0_MIX_WEIGHT: u32 = {int(p['ORDER0_MIX_WEIGHT'])};

/// Indirect / record model hash table bits.
pub const INDIRECT_BITS: usize = {int(p['INDIRECT_BITS'])};
/// Blend weight (0..=255) of indirect model into CM mix.
pub const INDIRECT_MIX_WEIGHT: u32 = {int(p['INDIRECT_MIX_WEIGHT'])};

/// Blend weight (0..=255) of match×indirect cross model into CM mix.
pub const MATCH_IND_CROSS_WEIGHT: u32 = {int(p['MATCH_IND_CROSS_WEIGHT'])};

/// Third-stage APM blend weight (0..=255) into final p after SSE2.
pub const APM_MIX_WEIGHT: u32 = {int(p['APM_MIX_WEIGHT'])};
/// APM adaptation shift (larger = slower). Careful default.
pub const APM_RATE_SHIFT: u32 = {int(p['APM_RATE_SHIFT'])};
// === END TUNABLE ==="""


def write_params(p: dict) -> None:
    merged = dict(DEFAULTS)
    merged.update(p)
    text = PARAMS.read_text()
    block = render(merged)
    new, n = BLOCK_RE.subn(block, text, count=1)
    if n != 1:
        sys.exit(f"write_params_full: expected 1 TUNABLE block, found {n}")
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
    write_params(p)
    merged = dict(DEFAULTS)
    merged.update(p)
    print(json.dumps(merged, separators=(",", ":")))


if __name__ == "__main__":
    main()
