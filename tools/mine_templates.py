#!/usr/bin/env python3
"""Mine MediaWiki {{template}} skeletons for PROMETHEUS TEMPLATE-MINER.

Reads a corpus (prefer post-capitals stream, or raw enwik), extracts
{{...}} skeletons with placeholders for variable args, ranks by estimated
net gain, and emits generated/template_dict.bin (+ report).

Usage:
  nix develop -c python3 tools/mine_templates.py enwik8 [--max 64] [--post-capitals]
  nix develop -c python3 tools/mine_templates.py enwik9 --max 64
"""

from __future__ import annotations

import argparse
import json
import re
import struct
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
if (ROOT / "ORCH_HALT").exists():
    print("HALTED by orch", flush=True)
    sys.exit(0)

GEN = ROOT / "generated"

# Capitals markers (must match capitals.rs) — used when --post-capitals is off
# and we simulate the post-capitals stream lightly for mining only.
CAP_SPACE = 0x01
CAP_NL = 0x02
CAP_GT = 0x03
CAP_DQUOTE = 0x04
CAP_SQUOTE = 0x05
CAP_LBRACK = 0x06
CAP_LPAREN = 0x07
CAP_BOS = 0x08
CAP_ESC = 0x7F

MAGIC = b"TMPL"


def encode_capitals_py(data: bytes) -> bytes:
    """Mirror of capitals.rs encode_capitals (for mining on post-capitals stream)."""
    out = bytearray()
    sep_map = {
        ord(" "): CAP_SPACE,
        ord("\n"): CAP_NL,
        ord(">"): CAP_GT,
        ord('"'): CAP_DQUOTE,
        ord("'"): CAP_SQUOTE,
        ord("["): CAP_LBRACK,
        ord("("): CAP_LPAREN,
    }
    for c in data:
        if ord("A") <= c <= ord("Z"):
            if not out:
                out.append(CAP_BOS)
                out.append(c + 32)
                continue
            prev = out[-1]
            if prev in sep_map:
                out[-1] = sep_map[prev]
                out.append(c + 32)
                continue
        if c in (
            CAP_SPACE,
            CAP_NL,
            CAP_GT,
            CAP_DQUOTE,
            CAP_SQUOTE,
            CAP_LBRACK,
            CAP_LPAREN,
            CAP_BOS,
            CAP_ESC,
        ):
            out.append(CAP_ESC)
            out.append(c)
        else:
            out.append(c)
    return bytes(out)


def extract_templates(data: bytes) -> list[tuple[int, int, bytes]]:
    """Return list of (start, end, inner_bytes) for top-level {{...}} spans."""
    out: list[tuple[int, int, bytes]] = []
    i = 0
    n = len(data)
    while i + 1 < n:
        if data[i] == ord("{") and data[i + 1] == ord("{"):
            depth = 1
            j = i + 2
            while j + 1 < n:
                if data[j] == ord("{") and data[j + 1] == ord("{"):
                    depth += 1
                    j += 2
                    continue
                if data[j] == ord("}") and data[j + 1] == ord("}"):
                    depth -= 1
                    j += 2
                    if depth == 0:
                        break
                    continue
                j += 1
            if depth == 0:
                inner = data[i + 2 : j - 2]
                out.append((i, j, inner))
                i = j
                continue
        i += 1
    return out


_PARAM_RE = re.compile(
    rb"(?:"
    rb"https?://[^\s|}{\[\]]+"  # URL
    rb"|www\.[^\s|}{\[\]]+"
    rb"|\d{1,}"  # numbers
    rb")"
)


def skeletonize(inner: bytes) -> tuple[bytes, int]:
    """Replace variable-looking pipe-args / URLs / numbers with \x00 placeholders.

    Returns (skeleton_with_nul_placeholders, n_params).
    Keeps template name (before first |) literal when short enough.
    """
    if not inner:
        return inner, 0
    # Split on top-level | (not inside nested {{}})
    parts: list[bytes] = []
    depth = 0
    start = 0
    i = 0
    while i < len(inner):
        if i + 1 < len(inner) and inner[i : i + 2] == b"{{":
            depth += 1
            i += 2
            continue
        if i + 1 < len(inner) and inner[i : i + 2] == b"}}":
            depth = max(0, depth - 1)
            i += 2
            continue
        if inner[i] == ord("|") and depth == 0:
            parts.append(inner[start:i])
            start = i + 1
        i += 1
    parts.append(inner[start:])

    skel_parts: list[bytes] = []
    n_params = 0
    for pi, part in enumerate(parts):
        if pi == 0:
            # Template name: keep as-is (normalize whitespace lightly)
            name = part.strip()[:64]
            skel_parts.append(name)
            continue
        # Named args: key=value → key=\x00 if value looks variable
        eq = part.find(b"=")
        if eq > 0 and eq < 40 and b"{" not in part[:eq]:
            key = part[:eq]
            val = part[eq + 1 :]
            if _is_variable(val):
                skel_parts.append(key + b"=\x00")
                n_params += 1
            else:
                skel_parts.append(part)
        elif _is_variable(part):
            skel_parts.append(b"\x00")
            n_params += 1
        else:
            # Still replace URLs/numbers inside otherwise-static args
            repl, k = _replace_vars(part)
            skel_parts.append(repl)
            n_params += k

    skel = b"|".join(skel_parts)
    return skel, n_params


def _is_variable(val: bytes) -> bool:
    v = val.strip()
    if not v:
        return True
    if len(v) > 48:
        return True
    if _PARAM_RE.fullmatch(v):
        return True
    # Mostly digits / mixed case free text → variable
    letters = sum(1 for c in v if 65 <= c <= 90 or 97 <= c <= 122)
    digits = sum(1 for c in v if 48 <= c <= 57)
    if digits and digits >= letters:
        return True
    if letters >= 1 and len(v) <= 40:
        return True
    return False


def _replace_vars(part: bytes) -> tuple[bytes, int]:
    n = 0
    out = bytearray()
    i = 0
    while i < len(part):
        m = _PARAM_RE.match(part, i)
        if m:
            out.append(0)
            n += 1
            i = m.end()
        else:
            out.append(part[i])
            i += 1
    return bytes(out), n


def mine(
    data: bytes,
    max_templates: int = 64,
    min_count: int = 8,
    min_skel_len: int = 6,
    max_skel_len: int = 96,
) -> list[dict]:
    spans = extract_templates(data)
    # Count skeleton occurrences + total raw bytes covered
    counts: dict[bytes, dict] = defaultdict(lambda: {"count": 0, "raw_bytes": 0, "params": 0})
    occurrences: dict[bytes, list[tuple[int, int]]] = defaultdict(list)

    for start, end, inner in spans:
        skel, nparam = skeletonize(inner)
        if len(skel) < min_skel_len or len(skel) > max_skel_len:
            continue
        # Drop skeletons that are all placeholders
        if skel.count(0) == len(skel):
            continue
        full = b"{{" + skel + b"}}"
        counts[full]["count"] += 1
        counts[full]["raw_bytes"] += end - start
        counts[full]["params"] = nparam
        occurrences[full].append((start, end))

    # Greedy cover: largest estimated gain first, mark covered spans
    covered = bytearray(len(data))

    def est_gain(skel: bytes, info: dict) -> float:
        # Rough: each occurrence saves (raw_len - 2) bytes of model input,
        # minus dict entry cost and ~1 byte token + params.
        # Prefer measured cost when a cost-log is supplied; here use raw heuristic.
        c = info["count"]
        avg_raw = info["raw_bytes"] / max(c, 1)
        token_cost = 2.0  # escape + index
        param_cost = info["params"] * 4.0  # rough avg param bytes still emitted
        dict_cost = len(skel) + 4
        per_hit = max(0.0, avg_raw - token_cost - param_cost)
        return per_hit * c - dict_cost

    ranked = sorted(counts.items(), key=lambda kv: est_gain(kv[0], kv[1]), reverse=True)
    selected: list[dict] = []
    for skel, info in ranked:
        if info["count"] < min_count:
            continue
        # Cover check: count uncovered occurrences
        occ = occurrences[skel]
        fresh = [(s, e) for s, e in occ if not any(covered[s:e])]
        if len(fresh) < min_count:
            continue
        for s, e in fresh:
            covered[s:e] = b"\x01" * (e - s)
        gain = est_gain(skel, {**info, "count": len(fresh), "raw_bytes": sum(e - s for s, e in fresh)})
        selected.append(
            {
                "skeleton": skel,
                "count": len(fresh),
                "params": info["params"],
                "est_gain": gain,
                "skel_len": len(skel),
            }
        )
        if len(selected) >= max_templates:
            break

    return selected


def write_blob(templates: list[dict], path: Path) -> None:
    """Binary: magic TMPL | u32 count | for each: u8 len, len bytes skeleton."""
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("wb") as f:
        f.write(MAGIC)
        f.write(struct.pack("<I", len(templates)))
        for t in templates:
            sk = t["skeleton"]
            if len(sk) > 255:
                sk = sk[:255]
            f.write(struct.pack("<B", len(sk)))
            f.write(sk)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("corpus", help="path to enwik8/enwik9 or post-capitals dump")
    ap.add_argument("--max", type=int, default=64)
    ap.add_argument("--min-count", type=int, default=8)
    ap.add_argument("--post-capitals", action="store_true", help="corpus already post-capitals")
    ap.add_argument("--apply-capitals", action="store_true", help="run capitals fold before mining")
    ap.add_argument("--out", type=Path, default=GEN / "template_dict.bin")
    args = ap.parse_args()

    path = Path(args.corpus)
    if not path.is_file():
        # allow bare enwik8 in repo root
        alt = ROOT / args.corpus
        if alt.is_file():
            path = alt
        else:
            print(f"missing corpus: {args.corpus}", file=sys.stderr)
            return 2

    print(f"loading {path} …", flush=True)
    data = path.read_bytes()
    if args.apply_capitals and not args.post_capitals:
        print("applying capitals fold …", flush=True)
        data = encode_capitals_py(data)
    print(f"mining on {len(data)} bytes …", flush=True)
    selected = mine(data, max_templates=args.max, min_count=args.min_count)
    write_blob(selected, args.out)

    report = {
        "corpus": str(path),
        "bytes": len(data),
        "n_templates": len(selected),
        "templates": [
            {
                "skeleton": t["skeleton"].decode("latin1", errors="replace"),
                "count": t["count"],
                "params": t["params"],
                "est_gain": t["est_gain"],
                "skel_len": t["skel_len"],
            }
            for t in selected
        ],
    }
    report_path = args.out.with_suffix(".json")
    report_path.write_text(json.dumps(report, indent=2))
    txt_path = args.out.with_suffix(".txt")
    with txt_path.open("w") as f:
        for i, t in enumerate(selected):
            sk = t["skeleton"].decode("latin1", errors="replace").replace("\x00", "·")
            f.write(f"{i:3d} count={t['count']:6d} params={t['params']} gain={t['est_gain']:.0f}  {sk}\n")

    print(f"wrote {args.out} ({args.out.stat().st_size} bytes), {len(selected)} templates")
    print(f"report {report_path}")
    if selected:
        print("top5:")
        for t in selected[:5]:
            sk = t["skeleton"].decode("latin1", errors="replace").replace("\x00", "·")
            print(f"  count={t['count']} gain={t['est_gain']:.0f} {sk[:80]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
