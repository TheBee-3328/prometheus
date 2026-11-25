//! Word-initial capitalization fold + reversible escape for marker collisions.

/// Escape prefix for literal cap-marker bytes and for `CAP_ESC` itself.
/// Chosen as `0x7F` (DEL): absent in enwik8, so SCORE-neutral on the corpus.
pub const CAP_ESC: u8 = 0x7F;

/// Capitalization markers (replace a preceding separator).
pub const CAP_SPACE: u8 = 0x01;
pub const CAP_NL: u8 = 0x02;
pub const CAP_GT: u8 = 0x03;
pub const CAP_DQUOTE: u8 = 0x04;
pub const CAP_SQUOTE: u8 = 0x05;
pub const CAP_LBRACK: u8 = 0x06;
pub const CAP_LPAREN: u8 = 0x07;
pub const CAP_BOS: u8 = 0x08; // capitalize next; no separator emitted

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
pub fn is_cap_marker(b: u8) -> bool {
    matches!(
        b,
        CAP_SPACE | CAP_NL | CAP_GT | CAP_DQUOTE | CAP_SQUOTE | CAP_LBRACK | CAP_LPAREN | CAP_BOS
    )
}

/// Fold word-initial `A..=Z` to lowercase; mark the preceding separator (or BOS).
/// Literal `0x01..=0x08` and `CAP_ESC` are escaped with a `CAP_ESC` prefix.
pub fn encode_capitals(input: &[u8]) -> Vec<u8> {
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
        if is_cap_marker(c) || c == CAP_ESC {
            out.push(CAP_ESC);
            out.push(c);
        } else {
            out.push(c);
        }
    }
    out
}

/// Inverse of [`encode_capitals`].
pub fn decode_capitals(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut capitalize = false;
    let mut i = 0;
    while i < input.len() {
        let c = input[i];
        i += 1;
        if capitalize {
            capitalize = false;
            if (b'a'..=b'z').contains(&c) {
                out.push(c - 32);
            } else {
                out.push(c);
            }
            continue;
        }
        if c == CAP_ESC {
            if i < input.len() {
                out.push(input[i]);
                i += 1;
            } else {
                // Truncated escape: emit the escape byte itself.
                out.push(CAP_ESC);
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
