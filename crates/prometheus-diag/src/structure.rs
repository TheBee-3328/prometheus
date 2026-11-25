//! Structural byte labeling + compressed-cost attribution (Track 0 / TEMPLATE-MINER).
//! Labels raw MediaWiki structure; propagates through the production preprocessor
//! (capitals → markup → words; templates OFF on main).

use prometheus_core::preprocessor::capitals::{
    is_cap_marker, CAP_BOS, CAP_DQUOTE, CAP_ESC, CAP_GT, CAP_LBRACK, CAP_LPAREN, CAP_NL, CAP_SPACE,
    CAP_SQUOTE,
};
use prometheus_core::preprocessor::markup::{DICT, DICT_BASE, ESC};
use prometheus_core::preprocessor::stages::{PRE_STAGE_CAPITALS, PRE_STAGE_MARKUP, PRE_STAGE_WORDS};
use prometheus_core::preprocessor::words::encode_words_labeled;
use prometheus_core::{encode, ArithmeticEncoder, Model, Order2MatchModel, MAGIC};
use std::io::{self, Write};

pub const L_OTHER: u8 = 0;
pub const L_XML: u8 = 1;
pub const L_ENTITY: u8 = 2;
pub const L_TMPL: u8 = 3;
pub const L_URL: u8 = 4;

pub const LABEL_NAMES: [&str; 5] = ["other", "xml_wrapper", "entity_tag", "template", "url"];

/// MediaWiki dump page-wrapper tags (and close tags) — structural XML chrome.
const XML_WRAPPERS: &[&[u8]] = &[
    b"<text xml:space=\"preserve\">",
    b"</contributor>",
    b"<contributor>",
    b"</timestamp>",
    b"<timestamp>",
    b"</revision>",
    b"<revision>",
    b"</username>",
    b"<username>",
    b"</comment>",
    b"<comment>",
    b"</title>",
    b"<title>",
    b"</page>",
    b"<page>",
    b"</text>",
    b"</id>",
    b"<id>",
    b"<minor />",
    b"<mediawiki",
    b"</mediawiki>",
    b"<siteinfo>",
    b"</siteinfo>",
    b"<namespaces>",
    b"</namespaces>",
    b"<namespace",
    b"</namespace>",
];

fn sep_to_cap_marker(sep: u8) -> Option<u8> {
    match sep {
        b' ' => Some(CAP_SPACE),
        b'\n' => Some(CAP_NL),
        b'>' => Some(CAP_GT),
        b'"' => Some(CAP_DQUOTE),
        b'\'' => Some(CAP_SQUOTE),
        b'[' => Some(CAP_LBRACK),
        b'(' => Some(CAP_LPAREN),
        _ => None,
    }
}

/// Prefer structural label when collapsing a matched span.
fn prefer_struct(a: u8, b: u8) -> u8 {
    for &cand in &[L_TMPL, L_ENTITY, L_URL, L_XML] {
        if a == cand || b == cand {
            return cand;
        }
    }
    a
}

/// Label each raw byte: xml_wrapper / entity_tag / {{template}} / URL / other.
pub fn label_structure(raw: &[u8]) -> Vec<u8> {
    let mut lab = vec![L_OTHER; raw.len()];

    // (a) XML page wrapper tags (longest-first).
    let mut xml_pats: Vec<&[u8]> = XML_WRAPPERS.to_vec();
    xml_pats.sort_by_key(|p| std::cmp::Reverse(p.len()));
    let mut i = 0usize;
    while i < raw.len() {
        let mut hit = false;
        for pat in &xml_pats {
            let n = pat.len();
            if i + n <= raw.len() && &raw[i..i + n] == *pat {
                let mut end = i + n;
                if *pat == b"<mediawiki" || *pat == b"<namespace" {
                    while end < raw.len() && raw[end] != b'>' {
                        end += 1;
                    }
                    if end < raw.len() {
                        end += 1;
                    }
                }
                for x in lab.iter_mut().take(end).skip(i) {
                    *x = L_XML;
                }
                i = end;
                hit = true;
                break;
            }
        }
        if !hit {
            i += 1;
        }
    }

    // (d) URLs
    i = 0;
    while i < raw.len() {
        let rest = &raw[i..];
        let start = if rest.starts_with(b"https://") {
            Some(8)
        } else if rest.starts_with(b"http://") {
            Some(7)
        } else if rest.starts_with(b"www.") {
            Some(4)
        } else {
            None
        };
        if let Some(prefix) = start {
            let mut j = i + prefix;
            while j < raw.len() {
                let c = raw[j];
                if c.is_ascii_whitespace()
                    || matches!(c, b'|' | b']' | b'"' | b'\'' | b'<' | b'>' | b'}' | b'{' | b'[')
                {
                    break;
                }
                j += 1;
            }
            for x in lab.iter_mut().take(j).skip(i) {
                if *x == L_OTHER {
                    *x = L_URL;
                }
            }
            i = j;
            continue;
        }
        i += 1;
    }

    // (b) entity-escaped HTML tags: &lt; ... &gt;
    i = 0;
    while i + 4 <= raw.len() {
        if &raw[i..i + 4] == b"&lt;" {
            let mut j = i + 4;
            let limit = (i + 512).min(raw.len());
            let mut found = None;
            while j + 4 <= limit {
                if &raw[j..j + 4] == b"&gt;" {
                    found = Some(j + 4);
                    break;
                }
                j += 1;
            }
            if let Some(end) = found {
                for x in lab.iter_mut().take(end).skip(i) {
                    if *x == L_OTHER || *x == L_URL {
                        *x = L_ENTITY;
                    }
                }
                i = end;
                continue;
            }
        }
        i += 1;
    }

    // (c) {{templates}} with nesting; overwrites inner labels.
    i = 0;
    while i + 1 < raw.len() {
        if raw[i] == b'{' && raw[i + 1] == b'{' {
            let mut depth = 1i32;
            let mut j = i + 2;
            while j + 1 < raw.len() {
                if raw[j] == b'{' && raw[j + 1] == b'{' {
                    depth += 1;
                    j += 2;
                    continue;
                }
                if raw[j] == b'}' && raw[j + 1] == b'}' {
                    depth -= 1;
                    j += 2;
                    if depth == 0 {
                        break;
                    }
                    continue;
                }
                j += 1;
            }
            if depth == 0 {
                for x in lab.iter_mut().take(j).skip(i) {
                    *x = L_TMPL;
                }
                i = j;
                continue;
            }
        }
        i += 1;
    }

    lab
}

fn encode_capitals_labels(input: &[u8], labels: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut out = Vec::with_capacity(input.len() + input.len() / 64);
    let mut out_l = Vec::with_capacity(out.capacity());
    for (idx, &c) in input.iter().enumerate() {
        let lab = labels[idx];
        if (b'A'..=b'Z').contains(&c) {
            if out.is_empty() {
                out.push(CAP_BOS);
                out_l.push(lab);
                out.push(c + 32);
                out_l.push(lab);
                continue;
            }
            let prev = *out.last().unwrap();
            if let Some(marker) = sep_to_cap_marker(prev) {
                *out.last_mut().unwrap() = marker;
                out.push(c + 32);
                out_l.push(lab);
                continue;
            }
        }
        if is_cap_marker(c) || c == CAP_ESC {
            out.push(CAP_ESC);
            out_l.push(lab);
            out.push(c);
            out_l.push(lab);
        } else {
            out.push(c);
            out_l.push(lab);
        }
    }
    (out, out_l)
}

fn encode_dict_labels(input: &[u8], labels: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut out = Vec::with_capacity(input.len());
    let mut out_l = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        let mut matched = false;
        for (idx, pat) in DICT.iter().enumerate() {
            let n = pat.len();
            if i + n <= input.len() && &input[i..i + n] == *pat {
                let mut lab = labels[i];
                for &l in &labels[i..i + n] {
                    lab = prefer_struct(lab, l);
                }
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
        let lab = labels[i];
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
    (out, out_l)
}

/// Encode with preprocessor, propagating structural labels onto the model stream.
pub fn encode_with_labels(raw: &[u8], labels: &[u8]) -> (Vec<u8>, Vec<u8>) {
    assert_eq!(raw.len(), labels.len());
    let mut cur = raw.to_vec();
    let mut lab = labels.to_vec();
    if PRE_STAGE_CAPITALS {
        let (c, l) = encode_capitals_labels(&cur, &lab);
        cur = c;
        lab = l;
    }
    if PRE_STAGE_MARKUP {
        let (c, l) = encode_dict_labels(&cur, &lab);
        cur = c;
        lab = l;
    }
    if PRE_STAGE_WORDS {
        let (c, l) = encode_words_labeled(&cur, Some(&lab));
        cur = c;
        lab = l;
    }
    // Prefer production byte stream if our mirror drifts.
    let prod = encode(raw);
    if cur != prod {
        let mut lab2 = lab;
        lab2.resize(prod.len(), L_OTHER);
        return (prod, lab2);
    }
    (cur, lab)
}

fn bit_cost_mbits(bit: u8, p1: u16) -> u32 {
    let p = if bit == 0 {
        65536u32.saturating_sub(p1 as u32)
    } else {
        p1 as u32
    };
    let p = p.max(1);
    let lz = p.leading_zeros();
    let log2_floor = 31 - lz;
    let frac_num = p - (1u32 << log2_floor);
    let frac_x1000 = if log2_floor == 0 {
        0
    } else {
        (frac_num * 1000) / (1u32 << log2_floor)
    };
    let log2_x1000 = log2_floor * 1000 + (frac_x1000 * 1443) / 1000;
    16000u32.saturating_sub(log2_x1000)
}

pub struct StructureReport {
    pub input_len: u64,
    pub pre_len: u64,
    pub labels_mirrored: bool,
    pub raw_counts: [u64; 5],
    pub pre_counts: [u64; 5],
    pub cost_mbits: [u64; 5],
    pub total_cost_mbits: u64,
}

impl StructureReport {
    pub fn compressed_bytes(&self, label: usize) -> f64 {
        self.cost_mbits[label] as f64 / 8000.0
    }

    pub fn total_compressed_bytes(&self) -> f64 {
        self.total_cost_mbits as f64 / 8000.0
    }

    pub fn share_pct(&self, label: usize) -> f64 {
        let t = self.total_cost_mbits;
        if t == 0 {
            0.0
        } else {
            100.0 * self.cost_mbits[label] as f64 / t as f64
        }
    }

    pub fn abcd_share_pct(&self) -> f64 {
        let t = self.total_cost_mbits;
        if t == 0 {
            0.0
        } else {
            let s = self.cost_mbits[L_XML as usize]
                + self.cost_mbits[L_ENTITY as usize]
                + self.cost_mbits[L_TMPL as usize]
                + self.cost_mbits[L_URL as usize];
            100.0 * s as f64 / t as f64
        }
    }

    pub fn print(&self) {
        println!(
            "STRUCTURE input_len={} pre_len={} labels_mirrored={} total_cost_mbits={} est_compressed_bytes={:.0}",
            self.input_len,
            self.pre_len,
            self.labels_mirrored as u8,
            self.total_cost_mbits,
            self.total_compressed_bytes()
        );
        for i in 0..5 {
            println!(
                "STRUCT {} raw_bytes={} pre_bytes={} cost_mbits={} est_bytes={:.0} share_pct={:.4}",
                LABEL_NAMES[i],
                self.raw_counts[i],
                self.pre_counts[i],
                self.cost_mbits[i],
                self.compressed_bytes(i),
                self.share_pct(i)
            );
        }
        let abcd = self.cost_mbits[1] + self.cost_mbits[2] + self.cost_mbits[3] + self.cost_mbits[4];
        println!(
            "STRUCT_ABCD cost_mbits={} est_bytes={:.0} share_pct={:.4} gate_3pct={}",
            abcd,
            abcd as f64 / 8000.0,
            self.abcd_share_pct(),
            if self.abcd_share_pct() >= 3.0 {
                "PASS"
            } else {
                "FAIL"
            }
        );
    }
}

/// Full-model structure-stats (preprocessor on). Writes a minimal archive to `out`.
pub fn run_structure_stats<W: Write>(raw: &[u8], mut out: W) -> io::Result<StructureReport> {
    eprintln!("structure-stats: labeling {} raw bytes…", raw.len());
    let labels = label_structure(raw);
    let mut raw_counts = [0u64; 5];
    for &l in &labels {
        raw_counts[l as usize] += 1;
    }

    eprintln!("structure-stats: preprocessing + label propagate…");
    let (data, pre_lab) = encode_with_labels(raw, &labels);
    let labels_mirrored = {
        let prod = encode(raw);
        data == prod && pre_lab.len() == data.len()
    };
    let mut pre_counts = [0u64; 5];
    for &l in &pre_lab {
        pre_counts[(l as usize).min(4)] += 1;
    }
    eprintln!(
        "structure-stats: pre_len={} labels_ok={} mirrored={}",
        data.len(),
        pre_lab.len() == data.len(),
        labels_mirrored
    );

    out.write_all(MAGIC)?;
    out.write_all(&(data.len() as u64).to_le_bytes())?;

    eprintln!("structure-stats: allocating Order2MatchModel…");
    let mut model = Order2MatchModel::new();
    eprintln!("structure-stats: compressing…");
    let mut enc = ArithmeticEncoder::new(out);
    let mut cost_mbits = [0u64; 5];
    let mut total = 0u64;

    for (pos, &byte) in data.iter().enumerate() {
        if pos > 0 && pos % 5_000_000 == 0 {
            eprintln!("structure-stats: byte {}/{}", pos, data.len());
        }
        let lab = *pre_lab.get(pos).unwrap_or(&L_OTHER) as usize;
        let lab = lab.min(4);
        for bi in (0..8).rev() {
            let bit = (byte >> bi) & 1;
            let p = model.predict();
            let c = bit_cost_mbits(bit, p) as u64;
            cost_mbits[lab] += c;
            total += c;
            enc.encode_bit(bit, p)?;
            model.update(bit);
        }
    }
    let _ = enc.finish()?;
    eprintln!("structure-stats: done total_cost_mbits={total}");

    Ok(StructureReport {
        input_len: raw.len() as u64,
        pre_len: data.len() as u64,
        labels_mirrored,
        raw_counts,
        pre_counts,
        cost_mbits,
        total_cost_mbits: total,
    })
}
