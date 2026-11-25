//! Mined MediaWiki `{{template}}` skeletons merged with static markup DICT.
//!
//! Blob (`generated/template_dict.bin`):
//!   magic b"TMPL" (4) | u32 LE count | for each: u8 len, `len` skeleton bytes
//! Skeletons use `0x00` as a parameter placeholder; literals are everything else.
//!
//! Token coding (post-capitals, shared pass with markup):
//!   `TMPL_ESC` (0xA3) + u8 index
//! Parameters: inline after the token (`PARAM_INLINE`) or a trailing side-stream.
//! Side-stream file layout: `u32 LE body_len` | body | params
//!   each param: `u16 BE len` | raw bytes (no escaping).

use std::sync::OnceLock;

use super::markup::{decode_dict, DICT, DICT_BASE, ESC};
use super::stages::{PARAM_INLINE, PRE_STAGE_MARKUP, PRE_STAGE_TEMPLATES, TEMPLATE_DICT_N};

/// Lead byte for template tokens. Sits at `markup_dict_end()` (gap before WORD_BASE).
pub const TMPL_ESC: u8 = 0xA3;

static TMPL_BLOB: &[u8] = include_bytes!("../../../../generated/template_dict.bin");

struct TemplateEntry {
    /// Literal segments separated by placeholders; len = n_params + 1.
    lits: Vec<Vec<u8>>,
    n_params: usize,
    /// Total skeleton byte length (for greedy longest-match tie-break).
    skel_len: usize,
}

static ENTRIES: OnceLock<Vec<TemplateEntry>> = OnceLock::new();

fn parse_blob(blob: &[u8]) -> Vec<TemplateEntry> {
    if blob.len() < 8 || &blob[0..4] != b"TMPL" {
        return Vec::new();
    }
    let count = u32::from_le_bytes([blob[4], blob[5], blob[6], blob[7]]) as usize;
    let mut out = Vec::new();
    let mut i = 8;
    while out.len() < count && i < blob.len() {
        let len = blob[i] as usize;
        i += 1;
        if i + len > blob.len() {
            break;
        }
        let skel = &blob[i..i + len];
        i += len;
        let mut lits: Vec<Vec<u8>> = Vec::new();
        let mut cur = Vec::new();
        let mut n_params = 0usize;
        for &b in skel {
            if b == 0 {
                lits.push(std::mem::take(&mut cur));
                n_params += 1;
            } else {
                cur.push(b);
            }
        }
        lits.push(cur);
        if TEMPLATE_DICT_N > 0 && out.len() >= TEMPLATE_DICT_N {
            break;
        }
        out.push(TemplateEntry {
            lits,
            n_params,
            skel_len: len,
        });
    }
    out
}

fn entries() -> &'static [TemplateEntry] {
    ENTRIES.get_or_init(|| {
        if !PRE_STAGE_TEMPLATES {
            return Vec::new();
        }
        parse_blob(TMPL_BLOB)
    })
}

/// Find `lit` in `input[start..]` only at brace-depth `== base_depth`.
fn find_lit_at_depth(input: &[u8], start: usize, lit: &[u8], base_depth: i32) -> Option<usize> {
    if lit.is_empty() {
        return Some(start);
    }
    let n = input.len();
    let mut depth = base_depth;
    let mut j = start;
    while j < n {
        // Prefer literal match at current depth before consuming braces
        // (so a closing `}}` lit matches at depth 1).
        if depth == base_depth && j + lit.len() <= n && &input[j..j + lit.len()] == lit {
            return Some(j);
        }
        if j + 1 < n && input[j] == b'{' && input[j + 1] == b'{' {
            depth += 1;
            j += 2;
            continue;
        }
        if j + 1 < n && input[j] == b'}' && input[j + 1] == b'}' {
            depth -= 1;
            j += 2;
            continue;
        }
        j += 1;
    }
    None
}

/// Try matching template `ti` at `pos`. Returns (params, end_exclusive).
fn match_template(input: &[u8], pos: usize, ti: &TemplateEntry) -> Option<(Vec<Vec<u8>>, usize)> {
    let lits = &ti.lits;
    if lits.is_empty() {
        return None;
    }
    let lit0 = &lits[0];
    if pos + lit0.len() > input.len() || &input[pos..pos + lit0.len()] != lit0.as_slice() {
        return None;
    }
    // After consuming opening literals that include `{{`, depth inside is 1.
    let mut cur = pos + lit0.len();
    let mut params: Vec<Vec<u8>> = Vec::with_capacity(ti.n_params);
    for next in lits.iter().skip(1) {
        let at = find_lit_at_depth(input, cur, next, 1)?;
        params.push(input[cur..at].to_vec());
        cur = at + next.len();
    }
    if params.len() != ti.n_params {
        return None;
    }
    Some((params, cur))
}

fn emit_param_inline(out: &mut Vec<u8>, param: &[u8]) {
    let len = param.len().min(0xffff);
    out.push((len >> 8) as u8);
    out.push((len & 0xff) as u8);
    for &b in &param[..len] {
        if b >= 0x80 || b == ESC || b == TMPL_ESC {
            out.push(ESC);
        }
        out.push(b);
    }
}

fn emit_param_side(side: &mut Vec<u8>, param: &[u8]) {
    let len = param.len().min(0xffff);
    side.push((len >> 8) as u8);
    side.push((len & 0xff) as u8);
    side.extend_from_slice(&param[..len]);
}

fn read_param_inline(input: &[u8], i: &mut usize) -> Option<Vec<u8>> {
    if *i + 2 > input.len() {
        return None;
    }
    let len = ((input[*i] as usize) << 8) | (input[*i + 1] as usize);
    *i += 2;
    let mut out = Vec::with_capacity(len);
    while out.len() < len && *i < input.len() {
        let b = input[*i];
        *i += 1;
        if b == ESC {
            if *i < input.len() {
                out.push(input[*i]);
                *i += 1;
            }
        } else {
            out.push(b);
        }
    }
    Some(out)
}

fn read_param_side(side: &[u8], i: &mut usize) -> Option<Vec<u8>> {
    if *i + 2 > side.len() {
        return None;
    }
    let len = ((side[*i] as usize) << 8) | (side[*i + 1] as usize);
    *i += 2;
    if *i + len > side.len() {
        return None;
    }
    let p = side[*i..*i + len].to_vec();
    *i += len;
    Some(p)
}

fn expand_template(ti: &TemplateEntry, params: &[Vec<u8>], out: &mut Vec<u8>) {
    for (k, lit) in ti.lits.iter().enumerate() {
        out.extend_from_slice(lit);
        if k < params.len() {
            out.extend_from_slice(&params[k]);
        }
    }
}

/// Same as [`encode_markup_templates`], optionally propagating labels onto the body.
/// Returns `(body, side_params, body_labels)`.
pub fn encode_markup_templates_labeled(
    input: &[u8],
    labels: Option<&[u8]>,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let tmpls = entries();
    let do_tmpl = PRE_STAGE_TEMPLATES && !tmpls.is_empty();
    let do_markup = PRE_STAGE_MARKUP;

    if !do_tmpl && !do_markup {
        let lab = labels
            .map(|l| l.to_vec())
            .unwrap_or_else(|| vec![0; input.len()]);
        return (input.to_vec(), Vec::new(), lab);
    }
    if !do_tmpl && do_markup {
        let mut out = Vec::with_capacity(input.len());
        let mut out_l = Vec::with_capacity(input.len());
        let mut i = 0;
        while i < input.len() {
            let mut matched = false;
            for (idx, pat) in DICT.iter().enumerate() {
                let n = pat.len();
                if i + n <= input.len() && &input[i..i + n] == *pat {
                    let lab = labels.map(|l| l[i]).unwrap_or(0);
                    out.push(DICT_BASE + idx as u8);
                    out_l.push(lab);
                    i += n;
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }
            let b = input[i];
            let lab = labels.map(|l| l[i]).unwrap_or(0);
            if b >= DICT_BASE {
                out.push(ESC);
                out_l.push(lab);
                out.push(b);
                out_l.push(lab);
            } else {
                out.push(b);
                out_l.push(lab);
            }
            i += 1;
        }
        return (out, Vec::new(), out_l);
    }

    let mut out = Vec::with_capacity(input.len());
    let mut out_l = Vec::with_capacity(input.len());
    let mut side = Vec::new();
    let mut i = 0;
    while i < input.len() {
        let lab = labels.map(|l| l[i]).unwrap_or(0);

        let mut best_tmpl: Option<(usize, usize, Vec<Vec<u8>>)> = None;
        for (ti, ent) in tmpls.iter().enumerate() {
            if let Some((params, end)) = match_template(input, i, ent) {
                let take = match &best_tmpl {
                    None => true,
                    Some((bti, bend, _)) => {
                        end > *bend
                            || (end == *bend && ent.skel_len > tmpls[*bti].skel_len)
                    }
                };
                if take {
                    best_tmpl = Some((ti, end, params));
                }
            }
        }

        let mut best_markup: Option<(usize, usize)> = None;
        if do_markup {
            for (idx, pat) in DICT.iter().enumerate() {
                let n = pat.len();
                if i + n <= input.len() && &input[i..i + n] == *pat {
                    if best_markup.map(|(_, l)| n > l).unwrap_or(true) {
                        best_markup = Some((idx, n));
                    }
                }
            }
        }

        let use_tmpl = match (&best_tmpl, &best_markup) {
            (Some((_, tend, _)), Some((_, mlen))) => *tend >= i + *mlen,
            (Some(_), None) => true,
            _ => false,
        };

        if use_tmpl {
            if let Some((ti, end, params)) = best_tmpl {
                out.push(TMPL_ESC);
                out.push(ti as u8);
                out_l.push(lab);
                out_l.push(lab);
                if PARAM_INLINE {
                    for p in &params {
                        emit_param_inline(&mut out, p);
                        while out_l.len() < out.len() {
                            out_l.push(lab);
                        }
                    }
                } else {
                    for p in &params {
                        emit_param_side(&mut side, p);
                    }
                }
                i = end;
                continue;
            }
        }

        if let Some((idx, n)) = best_markup {
            out.push(DICT_BASE + idx as u8);
            out_l.push(lab);
            i += n;
            continue;
        }

        let b = input[i];
        if b >= DICT_BASE {
            out.push(ESC);
            out_l.push(lab);
            out.push(b);
            out_l.push(lab);
        } else {
            out.push(b);
            out_l.push(lab);
        }
        i += 1;
    }
    (out, side, out_l)
}

/// Greedy longest-match: templates (full spans) then static markup DICT.
/// Returns `(body, side_params)` — `side_params` empty when `PARAM_INLINE`.
pub fn encode_markup_templates(input: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let (body, side, _) = encode_markup_templates_labeled(input, None);
    (body, side)
}

/// Inverse of the markup+templates body encoding. `side` is consumed when `!PARAM_INLINE`.
pub fn decode_markup_templates(input: &[u8], side: &[u8]) -> Vec<u8> {
    let tmpls = entries();
    let do_tmpl = PRE_STAGE_TEMPLATES && !tmpls.is_empty();
    let do_markup = PRE_STAGE_MARKUP;
    if !do_tmpl && !do_markup {
        return input.to_vec();
    }
    if !do_tmpl && do_markup {
        return decode_dict(input);
    }

    let dict_end = DICT_BASE + DICT.len() as u8;
    let mut out = Vec::with_capacity(input.len().saturating_mul(2));
    let mut i = 0;
    let mut si = 0usize;
    while i < input.len() {
        let b = input[i];
        if b == ESC {
            i += 1;
            if i < input.len() {
                out.push(input[i]);
                i += 1;
            }
            continue;
        }
        if do_tmpl && b == TMPL_ESC {
            i += 1;
            if i >= input.len() {
                out.push(TMPL_ESC);
                break;
            }
            let idx = input[i] as usize;
            i += 1;
            if idx >= tmpls.len() {
                out.push(TMPL_ESC);
                out.push(idx as u8);
                continue;
            }
            let ent = &tmpls[idx];
            let mut params = Vec::with_capacity(ent.n_params);
            for _ in 0..ent.n_params {
                let p = if PARAM_INLINE {
                    read_param_inline(input, &mut i).unwrap_or_default()
                } else {
                    read_param_side(side, &mut si).unwrap_or_default()
                };
                params.push(p);
            }
            expand_template(ent, &params, &mut out);
            continue;
        }
        if do_markup && b >= DICT_BASE && b < dict_end {
            out.extend_from_slice(DICT[(b - DICT_BASE) as usize]);
            i += 1;
            continue;
        }
        out.push(b);
        i += 1;
    }
    out
}

/// Wrap body + side params when using the side-stream layout.
pub fn pack_side_stream(body: &[u8], side: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + body.len() + side.len());
    let n = body.len() as u32;
    out.extend_from_slice(&n.to_le_bytes());
    out.extend_from_slice(body);
    out.extend_from_slice(side);
    out
}

/// Split side-stream layout; if templates off / inline / empty dict, body is whole input.
pub fn unpack_side_stream(input: &[u8]) -> (Vec<u8>, Vec<u8>) {
    if !PRE_STAGE_TEMPLATES || PARAM_INLINE || entries().is_empty() {
        return (input.to_vec(), Vec::new());
    }
    if input.len() < 4 {
        return (input.to_vec(), Vec::new());
    }
    let n = u32::from_le_bytes([input[0], input[1], input[2], input[3]]) as usize;
    if 4 + n > input.len() {
        return (input.to_vec(), Vec::new());
    }
    (input[4..4 + n].to_vec(), input[4 + n..].to_vec())
}

/// Whether encode/decode wrap the body with a u32 length prefix + side params.
#[inline]
pub fn uses_side_stream() -> bool {
    PRE_STAGE_TEMPLATES && !PARAM_INLINE && !entries().is_empty()
}

/// Parameter count for template index (words pass-through for inline params).
pub fn template_n_params(idx: usize) -> Option<usize> {
    entries().get(idx).map(|e| e.n_params)
}

/// Copy one length-prefixed escaped inline param from `input` to `out`.
pub fn copy_inline_param(input: &[u8], i: &mut usize, out: &mut Vec<u8>) {
    if *i + 2 > input.len() {
        return;
    }
    out.push(input[*i]);
    out.push(input[*i + 1]);
    let len = ((input[*i] as usize) << 8) | (input[*i + 1] as usize);
    *i += 2;
    let mut got = 0usize;
    while got < len && *i < input.len() {
        let b = input[*i];
        out.push(b);
        *i += 1;
        if b == ESC {
            if *i < input.len() {
                out.push(input[*i]);
                *i += 1;
            }
        }
        got += 1;
    }
}
