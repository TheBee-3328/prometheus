//! Natural-language word/phrase dictionary stage (XML-free).
//!
//! Binary blob format (little-endian):
//!   magic: b"WDCT" (4)
//!   count: u32
//!   for each entry: u8 len, then `len` bytes (ASCII)
//!
//! Entries are stored longest-first for greedy encode.
//! Token coding (post-markup stream):
//!   - indices `0..WORD_ONE_BYTE` → single byte `WORD_BASE + idx`
//!   - else → `WORD_ESC` + u16 BE index
//! Literal `WORD_ESC` is emitted as `WORD_ESC 0xFF WORD_ESC`.
//! Markup ESC pairs (`0xFF xx`) and markup tokens pass through unchanged.

use std::sync::OnceLock;

use super::markup::{markup_dict_end, DICT_BASE, ESC};
use super::stages::{PARAM_INLINE, PRE_STAGE_TEMPLATES, PRE_STAGE_WORDS, WORD_DICT_N};
use super::templates::{copy_inline_param, template_n_params, TMPL_ESC};

/// First single-byte word token (sits above static markup tokens).
pub const WORD_BASE: u8 = 0xA4;
/// Lead byte for multi-byte word tokens / WORD_ESC literal escape.
pub const WORD_ESC: u8 = 0xFE;
/// Number of words that use a single high byte (`WORD_BASE .. WORD_ESC`).
pub const WORD_ONE_BYTE: usize = (WORD_ESC - WORD_BASE) as usize; // 90

static WORD_DICT_BLOB: &[u8] = include_bytes!("../../../../generated/word_dict.bin");

static WORD_ENTRIES: OnceLock<Vec<Vec<u8>>> = OnceLock::new();
static WORD_TRIE: OnceLock<Trie> = OnceLock::new();

/// Compact trie: nodes[0] is root. `term[i] = Some(idx)` if node i ends a dict entry.
struct Trie {
    /// For each node: next[b] = child index, or 0 if none (0 = root, unused as child).
    next: Vec<[u32; 256]>,
    /// Entry index if this node is a terminal; u32::MAX if not.
    term: Vec<u32>,
}

impl Trie {
    fn new() -> Self {
        Self {
            next: vec![[0u32; 256]],
            term: vec![u32::MAX],
        }
    }

    fn insert(&mut self, pat: &[u8], idx: u32) {
        let mut node = 0u32;
        for &b in pat {
            let nxt = self.next[node as usize][b as usize];
            if nxt == 0 {
                let new_i = self.next.len() as u32;
                self.next.push([0u32; 256]);
                self.term.push(u32::MAX);
                self.next[node as usize][b as usize] = new_i;
                node = new_i;
            } else {
                node = nxt;
            }
        }
        // Longest-first insert order: first terminal wins for equal length;
        // greedy match walks full length so longer paths win naturally.
        if self.term[node as usize] == u32::MAX {
            self.term[node as usize] = idx;
        }
    }

    /// Longest *word-bounded* match at `input[pos..]`.
    /// Requires a non-letter (or BOS) before `pos`, and a non-letter (or EOS)
    /// after the match, so `the` does not fire inside `other`.
    fn longest_at(&self, input: &[u8], pos: usize) -> Option<(usize, usize)> {
        if pos > 0 && is_letter(input[pos - 1]) {
            return None;
        }
        let mut node = 0u32;
        let mut best: Option<(usize, usize)> = None;
        let mut len = 0usize;
        for &b in &input[pos..] {
            let nxt = self.next[node as usize][b as usize];
            if nxt == 0 {
                break;
            }
            node = nxt;
            len += 1;
            let t = self.term[node as usize];
            if t != u32::MAX {
                let end = pos + len;
                let right_ok = end >= input.len() || !is_letter(input[end]);
                if right_ok {
                    best = Some((t as usize, len));
                }
            }
        }
        best
    }
}

#[inline]
fn is_letter(b: u8) -> bool {
    matches!(b, b'a'..=b'z' | b'A'..=b'Z')
}

fn parse_blob(blob: &[u8]) -> Vec<Vec<u8>> {
    if blob.len() < 8 || &blob[0..4] != b"WDCT" {
        return Vec::new();
    }
    let count = u32::from_le_bytes([blob[4], blob[5], blob[6], blob[7]]) as usize;
    let mut entries = Vec::with_capacity(count.min(WORD_DICT_N.max(1)));
    let mut i = 8;
    while entries.len() < count && i < blob.len() {
        let len = blob[i] as usize;
        i += 1;
        if i + len > blob.len() {
            break;
        }
        entries.push(blob[i..i + len].to_vec());
        i += len;
        if WORD_DICT_N > 0 && entries.len() >= WORD_DICT_N {
            break;
        }
    }
    entries
}

fn entries() -> &'static [Vec<u8>] {
    WORD_ENTRIES.get_or_init(|| {
        if !PRE_STAGE_WORDS || WORD_DICT_N == 0 {
            return Vec::new();
        }
        parse_blob(WORD_DICT_BLOB)
    })
}

fn trie() -> &'static Trie {
    WORD_TRIE.get_or_init(|| {
        let mut t = Trie::new();
        for (idx, e) in entries().iter().enumerate() {
            t.insert(e, idx as u32);
        }
        t
    })
}

#[inline]
fn emit_word_token(out: &mut Vec<u8>, idx: usize) {
    if idx < WORD_ONE_BYTE {
        out.push(WORD_BASE + idx as u8);
    } else {
        out.push(WORD_ESC);
        out.push((idx >> 8) as u8);
        out.push((idx & 0xff) as u8);
    }
}

/// Greedy word/phrase substitution on the post-markup stream.
pub fn encode_words(input: &[u8]) -> Vec<u8> {
    encode_words_labeled(input, None).0
}

/// Same as [`encode_words`], optionally propagating per-byte labels onto the
/// output stream (first byte of a matched span / literal supplies the label).
pub fn encode_words_labeled(input: &[u8], labels: Option<&[u8]>) -> (Vec<u8>, Vec<u8>) {
    if !PRE_STAGE_WORDS || entries().is_empty() {
        let lab = labels.map(|l| l.to_vec()).unwrap_or_else(|| vec![0; input.len()]);
        return (input.to_vec(), lab);
    }
    let t = trie();
    let markup_end = markup_dict_end();
    let mut out = Vec::with_capacity(input.len());
    let mut out_l = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        let b = input[i];
        let lab = labels.map(|l| l[i]).unwrap_or(0);
        // Pass through markup ESC pairs unchanged.
        if b == ESC {
            out.push(ESC);
            out_l.push(lab);
            i += 1;
            if i < input.len() {
                let lab2 = labels.map(|l| l[i]).unwrap_or(0);
                out.push(input[i]);
                out_l.push(lab2);
                i += 1;
            }
            continue;
        }
        // Pass through static markup tokens.
        if b >= DICT_BASE && b < markup_end {
            out.push(b);
            out_l.push(lab);
            i += 1;
            continue;
        }
        // Pass through template tokens (and inline params when enabled).
        if PRE_STAGE_TEMPLATES && b == TMPL_ESC {
            out.push(TMPL_ESC);
            out_l.push(lab);
            i += 1;
            if i < input.len() {
                let idx_b = input[i];
                out.push(idx_b);
                out_l.push(lab);
                i += 1;
                if PARAM_INLINE {
                    if let Some(np) = template_n_params(idx_b as usize) {
                        for _ in 0..np {
                            let before = out.len();
                            copy_inline_param(input, &mut i, &mut out);
                            while out_l.len() < out.len() {
                                out_l.push(lab);
                            }
                            let _ = before;
                        }
                    }
                }
            }
            continue;
        }
        // Try dictionary match (natural-language only; patterns are ASCII).
        if let Some((idx, len)) = t.longest_at(input, i) {
            let before = out.len();
            emit_word_token(&mut out, idx);
            while out_l.len() < out.len() {
                out_l.push(lab);
            }
            debug_assert_eq!(out.len(), before + (out.len() - before));
            let _ = before;
            i += len;
            continue;
        }
        // Escape bare WORD_ESC (should be rare post-markup).
        if b == WORD_ESC {
            out.push(WORD_ESC);
            out.push(0xFF);
            out.push(WORD_ESC);
            out_l.push(lab);
            out_l.push(lab);
            out_l.push(lab);
            i += 1;
            continue;
        }
        // Escape bare single-byte word token range (rare).
        if (WORD_BASE..WORD_ESC).contains(&b) {
            out.push(WORD_ESC);
            out.push(0xFF);
            out.push(b);
            out_l.push(lab);
            out_l.push(lab);
            out_l.push(lab);
            i += 1;
            continue;
        }
        out.push(b);
        out_l.push(lab);
        i += 1;
    }
    (out, out_l)
}

/// Inverse of [`encode_words`].
pub fn decode_words(input: &[u8]) -> Vec<u8> {
    if !PRE_STAGE_WORDS || entries().is_empty() {
        return input.to_vec();
    }
    let ents = entries();
    let markup_end = markup_dict_end();
    let mut out = Vec::with_capacity(input.len().saturating_mul(2));
    let mut i = 0;
    while i < input.len() {
        let b = input[i];
        if b == ESC {
            out.push(ESC);
            i += 1;
            if i < input.len() {
                out.push(input[i]);
                i += 1;
            }
            continue;
        }
        if b >= DICT_BASE && b < markup_end {
            out.push(b);
            i += 1;
            continue;
        }
        if PRE_STAGE_TEMPLATES && b == TMPL_ESC {
            out.push(TMPL_ESC);
            i += 1;
            if i < input.len() {
                let idx_b = input[i];
                out.push(idx_b);
                i += 1;
                if PARAM_INLINE {
                    if let Some(np) = template_n_params(idx_b as usize) {
                        for _ in 0..np {
                            copy_inline_param(input, &mut i, &mut out);
                        }
                    }
                }
            }
            continue;
        }
        if (WORD_BASE..WORD_ESC).contains(&b) {
            let idx = (b - WORD_BASE) as usize;
            if idx < ents.len() {
                out.extend_from_slice(&ents[idx]);
            } else {
                out.push(b);
            }
            i += 1;
            continue;
        }
        if b == WORD_ESC {
            i += 1;
            if i >= input.len() {
                out.push(WORD_ESC);
                break;
            }
            let n1 = input[i];
            // Escape: WORD_ESC 0xFF <literal>
            if n1 == 0xFF {
                i += 1;
                if i < input.len() {
                    out.push(input[i]);
                    i += 1;
                }
                continue;
            }
            // Multi-byte index: WORD_ESC + u16 BE
            if i + 1 >= input.len() {
                out.push(WORD_ESC);
                out.push(n1);
                break;
            }
            let idx = ((n1 as usize) << 8) | (input[i + 1] as usize);
            i += 2;
            if idx < ents.len() {
                out.extend_from_slice(&ents[idx]);
            }
            continue;
        }
        out.push(b);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_blob_roundtrip_identity() {
        // With PRE_STAGE_WORDS default false, encode/decode are identity.
        let s = b"hello world the and";
        assert_eq!(encode_words(s), s);
        assert_eq!(decode_words(s), s);
    }
}
