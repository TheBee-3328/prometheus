//! Static XML/wiki markup dictionary (longest-match, high-byte tokens).

/// Escape prefix for literal bytes that collide with dictionary tokens / escape.
pub const ESC: u8 = 0xFF;

/// First dictionary token byte; tokens are `DICT_BASE .. DICT_BASE+DICT.len()`.
pub const DICT_BASE: u8 = 0x80;

/// Common enwik / MediaWiki strings, longest-first for greedy match.
/// Keep short: each entry is embedded in the decompressor.
pub static DICT: &[&[u8]] = &[
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
    b"#REDIRECT",
    b"Category:",
    b"http://",
    b"&quot;",
    b"&amp;",
    b"&lt;",
    b"&gt;",
    b"\n        ",
    b"\n      ",
    b"\n    ",
    b"\n  ",
    b"[[",
    b"]]",
    b"{{",
    b"}}",
    b"  ",
];

#[inline]
pub fn markup_dict_end() -> u8 {
    DICT_BASE + DICT.len() as u8
}

/// Greedy longest-match dictionary + high-byte escape.
pub fn encode_dict(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        let mut matched = false;
        for (idx, pat) in DICT.iter().enumerate() {
            let n = pat.len();
            if i + n <= input.len() && &input[i..i + n] == *pat {
                out.push(DICT_BASE + idx as u8);
                i += n;
                matched = true;
                break;
            }
        }
        if matched {
            continue;
        }
        let b = input[i];
        if b >= DICT_BASE {
            out.push(ESC);
            out.push(b);
        } else {
            out.push(b);
        }
        i += 1;
    }
    out
}

/// Inverse of [`encode_dict`].
pub fn decode_dict(input: &[u8]) -> Vec<u8> {
    let dict_end = markup_dict_end();
    let mut out = Vec::with_capacity(input.len().saturating_mul(2));
    let mut i = 0;
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
        if b >= DICT_BASE && b < dict_end {
            out.extend_from_slice(DICT[(b - DICT_BASE) as usize]);
            i += 1;
            continue;
        }
        out.push(b);
        i += 1;
    }
    out
}
