//! Reversible preprocessor for enwik-style XML/text.
//!
//! Two transforms (encode order; decode reverses):
//! 1. Word-initial capitalization → lowercase + marker on the preceding separator.
//! 2. Longest-match dictionary of common XML/wiki strings → single high bytes
//!    (`0x80..`), with `0xFF` escaping for literal high bytes.

/// Escape prefix for literal bytes that collide with dictionary tokens / escape.
const ESC: u8 = 0xFF;

/// First dictionary token byte; tokens are `DICT_BASE .. DICT_BASE+DICT.len()`.
const DICT_BASE: u8 = 0x80;

/// Capitalization markers (replace a preceding separator). Unused in raw enwik8.
const CAP_SPACE: u8 = 0x01;
const CAP_NL: u8 = 0x02;
const CAP_GT: u8 = 0x03;
const CAP_DQUOTE: u8 = 0x04;
const CAP_SQUOTE: u8 = 0x05;
const CAP_LBRACK: u8 = 0x06;
const CAP_LPAREN: u8 = 0x07;
const CAP_BOS: u8 = 0x08; // capitalize next; no separator emitted

/// Common enwik / MediaWiki strings, longest-first for greedy match.
/// Keep short: each entry is embedded in the decompressor.
static DICT: &[&[u8]] = &[
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

#[inline]
fn cap_marker_to_sep(marker: u8) -> Option<u8> {
    match marker {
        CAP_SPACE => Some(b' '),
        CAP_NL => Some(b'\n'),
        CAP_GT => Some(b'>'),
        CAP_DQUOTE => Some(b'"'),
        CAP_SQUOTE => Some(b'\''),
        CAP_LBRACK => Some(b'['),
        CAP_LPAREN => Some(b'('),
        CAP_BOS => None,
        _ => None,
    }
}

#[inline]
fn is_cap_marker(b: u8) -> bool {
    matches!(
        b,
        CAP_SPACE | CAP_NL | CAP_GT | CAP_DQUOTE | CAP_SQUOTE | CAP_LBRACK | CAP_LPAREN | CAP_BOS
    )
}

/// Fold word-initial `A..=Z` to lowercase; mark the preceding separator (or BOS).
fn encode_capitals(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() + input.len() / 64);
    for &c in input {
        if (b'A'..=b'Z').contains(&c) {
            if out.is_empty() {
                out.push(CAP_BOS);
                out.push(c + 32);
                continue;
            }
            let prev = *out.last().unwrap();
            if let Some(marker) = sep_to_cap_marker(prev) {
                *out.last_mut().unwrap() = marker;
                out.push(c + 32);
                continue;
            }
        }
        out.push(c);
    }
    out
}

/// Inverse of [`encode_capitals`].
fn decode_capitals(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut capitalize = false;
    for &c in input {
        if capitalize {
            capitalize = false;
            if (b'a'..=b'z').contains(&c) {
                out.push(c - 32);
            } else {
                out.push(c);
            }
            continue;
        }
        if is_cap_marker(c) {
            if let Some(sep) = cap_marker_to_sep(c) {
                out.push(sep);
            }
            capitalize = true;
            continue;
        }
        out.push(c);
    }
    out
}

/// Greedy longest-match dictionary + high-byte escape.
fn encode_dict(input: &[u8]) -> Vec<u8> {
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
fn decode_dict(input: &[u8]) -> Vec<u8> {
    let dict_end = DICT_BASE + DICT.len() as u8;
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

/// Encode: capitals then dictionary. Inverse of [`decode`].
pub fn encode(input: &[u8]) -> Vec<u8> {
    let capped = encode_capitals(input);
    encode_dict(&capped)
}

/// Decode: dictionary then capitals. Inverse of [`encode`].
pub fn decode(input: &[u8]) -> Vec<u8> {
    let undict = decode_dict(input);
    decode_capitals(&undict)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_roundtrip(data: &[u8]) {
        let enc = encode(data);
        let dec = decode(&enc);
        assert_eq!(dec, data, "roundtrip failed for {data:?}");
    }

    #[test]
    fn encode_decode_symmetry_xml_sample() {
        let sample = br#"<page>
  <title>AlgeriA</title>
  <id>5</id>
  <revision>
    <timestamp>2005-07-03T11:13:13Z</timestamp>
    <contributor>
      <username>Docu</username>
    </contributor>
    <text xml:space="preserve">Hello World &amp; [[Category:Test]] {{foo}} &quot;Ok&quot;</text>
  </revision>
</page>
"#;
        assert_roundtrip(sample);
    }

    #[test]
    fn encode_decode_empty_and_edge() {
        assert_roundtrip(b"");
        assert_roundtrip(b"A");
        assert_roundtrip(b" Hello");
        assert_roundtrip(b"\nWorld");
        assert_roundtrip(b">Title");
        assert_roundtrip(&[0x00, 0x80, 0xFF, 0xFE, b'A', b' ']);
        assert_roundtrip(b"[[link]] &lt;tag&gt;  spaced");
    }

    #[test]
    fn capitals_only_roundtrip() {
        let s = b"Alpha beta Gamma\nDelta >Epsilon \"Zeta\" 'Eta' [Theta] (Iota)";
        let e = encode_capitals(s);
        assert_eq!(decode_capitals(&e), s);
    }

    #[test]
    fn dict_shrinks_common_tags() {
        let s = b"<page><title>X</title></page>";
        let e = encode_dict(s);
        assert!(e.len() < s.len());
        assert_eq!(decode_dict(&e), s);
    }
}
