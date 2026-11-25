#!/usr/bin/env python3
"""Build XML-free natural-language word/phrase dictionaries from enwik9.

Two-pass, memory-capped:
  Pass 1 — stream text, strip tags/entities, count unigrams/bigrams/trigrams
           with periodic pruning so the counter stays under MAX_MAP_ENTRIES.
  Pass 2 — rescore surviving candidates on a second full scan; emit top-N blobs.

Usage:
  python3 tools/build_word_dict.py enwik9
  python3 tools/build_word_dict.py --select 4096
"""

from __future__ import annotations

import argparse
import os
import re
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GEN = ROOT / "generated"

SIZES = (4096, 16384, 49152)

# Markup / schema / wiki-boilerplate tokens — never put these in the NL word dict.
SCHEMA_BLACKLIST = {
    # dump / mediawiki schema
    "page",
    "revision",
    "timestamp",
    "contributor",
    "username",
    "comment",
    "title",
    "text",
    "id",
    "minor",
    "mediawiki",
    "xmlns",
    "schema",
    "sitename",
    "base",
    "generator",
    "case",
    "namespaces",
    "namespace",
    "redirect",
    "revisionid",
    "parentid",
    "format",
    "model",
    "sha1",
    "origin",
    "ip",
    "restrictions",
    # wiki / HTML boilerplate that is not natural language
    "category",
    "template",
    "wikipedia",
    "wiktionary",
    "wikimedia",
    "cellpadding",
    "cellspacing",
    "colspan",
    "rowspan",
    "bgcolor",
    "align",
    "valign",
    "border",
    "style",
    "class",
    "href",
    "http",
    "https",
    "www",
    "nbsp",
    "mdash",
    "ndash",
    "reflist",
    "refbegin",
    "refend",
    "cite",
    "citation",
    "infobox",
    "navbox",
    "stub",
    "disambiguation",
    "autowikibrowser",
    "multilicensewithcc",
    "populationpercent",
    "populationdensity",
    "populationincrease",
    "unincorporated",
    "townships",
    "municipalities",
    "coord",
    "coords",
    "image",
    "filename",
    "thumb",
    "upright",
    "special",
    "contributions",
    "user",
    "talk",
    "help",
    "file",
    "media",
    "module",
    "project",
}

# Memory caps (14GB host / shared machine: keep well under available RAM).
MAX_MAP_ENTRIES = 800_000
PRUNE_KEEP = 400_000
MIN_COUNT_START = 2
# Token byte cost model: first 90 entries → 1 byte; rest → 3 bytes.
WORD_ONE_BYTE = 90
TOKEN_COST_SHORT = 1
TOKEN_COST_LONG = 3

TAG_RE = re.compile(rb"<[^>]*>")
ENTITY_RE = re.compile(rb"&[A-Za-z0-9#]+;")
WORD_RE = re.compile(rb"[A-Za-z]+")


def strip_markup(chunk: bytes) -> bytes:
    chunk = TAG_RE.sub(b" ", chunk)
    chunk = ENTITY_RE.sub(b" ", chunk)
    return chunk


def is_schema(word: str) -> bool:
    return word in SCHEMA_BLACKLIST


def token_cost(rank: int) -> int:
    return TOKEN_COST_SHORT if rank < WORD_ONE_BYTE else TOKEN_COST_LONG


def savings(count: int, nbytes: int, rank: int) -> int:
    cost = token_cost(rank)
    if nbytes <= cost:
        return 0
    return count * (nbytes - cost)


def prune_counts(counts: dict[str, int], keep: int, min_count: int) -> dict[str, int]:
    """Drop low-count keys; if still too big, keep top-`keep` by count."""
    filtered = {k: v for k, v in counts.items() if v >= min_count}
    if len(filtered) <= keep:
        return filtered
    # Keep highest counts (stable-ish via count then key length).
    top = sorted(filtered.items(), key=lambda kv: (kv[1], len(kv[0])), reverse=True)[:keep]
    return dict(top)


def iter_words_from_file(path: Path, chunk_size: int = 8 * 1024 * 1024):
    """Yield lowercase word strings, streaming; carries overlap across chunks."""
    carry = b""
    with path.open("rb") as f:
        while True:
            raw = f.read(chunk_size)
            if not raw:
                break
            data = carry + strip_markup(raw)
            # Keep a tail so words/phrases spanning chunk boundaries survive.
            if len(data) > 64:
                body, carry = data[:-64], data[-64:]
            else:
                body, carry = data, b""
            for m in WORD_RE.finditer(body):
                w = m.group(0).decode("ascii", errors="ignore").lower()
                if len(w) >= 2 and not is_schema(w):
                    yield w
        # Flush carry
        if carry:
            for m in WORD_RE.finditer(strip_markup(carry)):
                w = m.group(0).decode("ascii", errors="ignore").lower()
                if len(w) >= 2 and not is_schema(w):
                    yield w


def pass1_count(path: Path) -> dict[str, int]:
    counts: dict[str, int] = {}
    min_count = MIN_COUNT_START
    n_words = 0
    t0 = time.time()
    prev2: str | None = None
    prev1: str | None = None

    def bump(key: str) -> None:
        counts[key] = counts.get(key, 0) + 1

    for w in iter_words_from_file(path):
        n_words += 1
        bump(w)
        if prev1 is not None:
            bump(prev1 + " " + w)
        if prev2 is not None and prev1 is not None:
            bump(prev2 + " " + prev1 + " " + w)
        prev2, prev1 = prev1, w

        if n_words & 0xFFFFF == 0:  # every ~1M words
            if len(counts) > MAX_MAP_ENTRIES:
                before = len(counts)
                counts = prune_counts(counts, PRUNE_KEEP, min_count)
                print(
                    f"  prune @ {n_words} words: {before} → {len(counts)} "
                    f"(min_count={min_count})",
                    flush=True,
                )
                if len(counts) > MAX_MAP_ENTRIES * 0.9:
                    min_count += 1
            elapsed = time.time() - t0
            print(
                f"  pass1 {n_words/1e6:.1f}M words, map={len(counts)}, "
                f"{elapsed:.0f}s",
                flush=True,
            )

    counts = prune_counts(counts, max(PRUNE_KEEP * 2, max(SIZES) * 4), min_count)
    print(
        f"pass1 done: {n_words} words, {len(counts)} candidates, "
        f"{time.time()-t0:.1f}s",
        flush=True,
    )
    return counts


def pass2_rescore(path: Path, candidates: set[str]) -> dict[str, int]:
    """Exact counts for candidate phrases only (second scan)."""
    tallies = {c: 0 for c in candidates}
    # Index by first word for fast phrase checks.
    by_first: dict[str, list[str]] = {}
    for c in candidates:
        first = c.split(" ", 1)[0]
        by_first.setdefault(first, []).append(c)
    # Longer phrases first when matching at a position.
    for lst in by_first.values():
        lst.sort(key=lambda s: (-s.count(" "), -len(s)))

    n_words = 0
    t0 = time.time()
    window: list[str] = []

    for w in iter_words_from_file(path):
        n_words += 1
        window.append(w)
        if len(window) > 3:
            window.pop(0)

        # Unigram
        if w in tallies:
            tallies[w] += 1
        # Bigram / trigram ending here
        if len(window) >= 2:
            bi = window[-2] + " " + window[-1]
            if bi in tallies:
                tallies[bi] += 1
        if len(window) >= 3:
            tri = window[-3] + " " + window[-2] + " " + window[-1]
            if tri in tallies:
                tallies[tri] += 1

        if n_words & 0xFFFFF == 0:
            print(
                f"  pass2 {n_words/1e6:.1f}M words, {time.time()-t0:.0f}s",
                flush=True,
            )

    print(f"pass2 done: {n_words} words, {time.time()-t0:.1f}s", flush=True)
    return tallies


def select_top(tallies: dict[str, int], n: int) -> list[str]:
    """Pick top-n by estimated byte savings.

    Blob order = savings rank (best first) so indices 0..89 get 1-byte tokens.
    Greedy longest-match is handled by the encoder trie, not blob order.
    """
    scored: list[tuple[int, int, str]] = []
    for phrase, cnt in tallies.items():
        raw = phrase.encode("ascii")
        if len(raw) > 255 or len(raw) < 3:
            continue
        parts = phrase.split()
        if any(is_schema(p) for p in parts):
            continue
        # Skip camel/glue monsters: long unigrams with mixed letter runs are rare NL.
        if len(parts) == 1 and len(raw) > 20:
            continue
        # Optimistic cost=1 for ranking seed; re-rank below.
        sc = cnt * max(0, len(raw) - TOKEN_COST_SHORT)
        if sc <= 0:
            continue
        scored.append((sc, len(raw), phrase))

    scored.sort(key=lambda t: (t[0], t[1]), reverse=True)
    # Re-score with true token cost by provisional rank.
    rescored: list[tuple[int, int, str]] = []
    for rank, (_, _, phrase) in enumerate(scored[: n * 3]):
        cnt = tallies[phrase]
        nbytes = len(phrase.encode("ascii"))
        sc = savings(cnt, nbytes, rank)
        if sc > 0:
            rescored.append((sc, nbytes, phrase))
    rescored.sort(key=lambda t: (t[0], t[1]), reverse=True)
    return [p for _, _, p in rescored[:n]]


def write_blob(phrases: list[str], dest: Path) -> int:
    parts = [b"WDCT", len(phrases).to_bytes(4, "little")]
    for p in phrases:
        raw = p.encode("ascii")
        if len(raw) > 255:
            continue
        parts.append(bytes([len(raw)]))
        parts.append(raw)
    data = b"".join(parts)
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_bytes(data)
    return len(data)


def build(enwik_path: Path) -> None:
    GEN.mkdir(parents=True, exist_ok=True)
    print(f"building word dicts from {enwik_path}", flush=True)
    counts = pass1_count(enwik_path)
    # Keep a generous candidate pool for pass2.
    pool_n = max(SIZES) * 3
    seed = sorted(counts.items(), key=lambda kv: (kv[1] * max(1, len(kv[0]) - 1), len(kv[0])), reverse=True)
    candidates = {k for k, _ in seed[:pool_n]}
    print(f"pass2 candidates: {len(candidates)}", flush=True)
    tallies = pass2_rescore(enwik_path, candidates)

    report_lines = [
        "# word_dict report",
        "",
        f"Source: `{enwik_path}`",
        f"Candidates after pass1 pool: {len(candidates)}",
        "",
        "| N | blob bytes | top phrase | top savings est. |",
        "|--:|----------:|:-----------|-----------------:|",
    ]
    for n in SIZES:
        phrases = select_top(tallies, n)
        dest = GEN / f"word_dict_{n}.bin"
        nbytes = write_blob(phrases, dest)
        top = phrases[0] if phrases else ""
        top_sav = savings(tallies.get(top, 0), len(top.encode("ascii")), 0) if top else 0
        report_lines.append(f"| {n} | {nbytes} | `{top}` | {top_sav} |")
        print(f"wrote {dest} ({nbytes} bytes, {len(phrases)} entries)", flush=True)
        # Sample lines
        sample_path = GEN / f"word_dict_{n}_sample.txt"
        sample_path.write_text("\n".join(phrases[:50]) + "\n")

    (GEN / "word_dict_report.md").write_text("\n".join(report_lines) + "\n")
    print(f"wrote {GEN / 'word_dict_report.md'}", flush=True)


def select_n(n: int) -> None:
    src = GEN / f"word_dict_{n}.bin"
    dst = GEN / "word_dict.bin"
    if not src.exists():
        print(f"missing {src}; run build first", file=sys.stderr)
        sys.exit(1)
    dst.write_bytes(src.read_bytes())
    print(f"selected N={n}: {src} → {dst} ({dst.stat().st_size} bytes)")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "enwik",
        nargs="?",
        help="path to enwik9 (or enwik8) for building",
    )
    ap.add_argument(
        "--select",
        type=int,
        choices=SIZES,
        help="copy generated/word_dict_N.bin → generated/word_dict.bin",
    )
    args = ap.parse_args()
    if args.select is not None:
        select_n(args.select)
        return
    if not args.enwik:
        ap.error("provide enwik path or --select N")
    path = Path(args.enwik)
    if not path.is_file():
        # allow bare 'enwik9' relative to repo root
        alt = ROOT / args.enwik
        if alt.is_file():
            path = alt
        else:
            print(f"not found: {args.enwik}", file=sys.stderr)
            sys.exit(1)
    build(path)


if __name__ == "__main__":
    main()
