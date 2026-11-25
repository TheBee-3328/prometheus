//! Reversible preprocessor for enwik-style XML/text.
//!
//! Encode order (decode reverses):
//! 1. Capitals fold (optional)
//! 2. Static markup dictionary (optional)
//! 3. Natural-language word/phrase dictionary (optional)
//!
//! Cap markers occupy `0x01..=0x08`. Literal bytes in that range (and the
//! escape byte `0x7F`) are prefixed with `CAP_ESC` so encode/decode are
//! inverses for every `&[u8]`.

pub mod capitals;
pub mod markup;
pub mod stages;
pub mod words;

use capitals::{decode_capitals, encode_capitals};
use markup::{decode_dict, encode_dict};
use stages::{PRE_STAGE_CAPITALS, PRE_STAGE_MARKUP, PRE_STAGE_WORDS};
use words::{decode_words, encode_words};

/// Encode: capitals → markup → words. Inverse of [`decode`].
pub fn encode(input: &[u8]) -> Vec<u8> {
    let mut cur = input.to_vec();
    if PRE_STAGE_CAPITALS {
        cur = encode_capitals(&cur);
    }
    if PRE_STAGE_MARKUP {
        cur = encode_dict(&cur);
    }
    if PRE_STAGE_WORDS {
        cur = encode_words(&cur);
    }
    cur
}

/// Decode: words → markup → capitals. Inverse of [`encode`].
pub fn decode(input: &[u8]) -> Vec<u8> {
    let mut cur = input.to_vec();
    if PRE_STAGE_WORDS {
        cur = decode_words(&cur);
    }
    if PRE_STAGE_MARKUP {
        cur = decode_dict(&cur);
    }
    if PRE_STAGE_CAPITALS {
        cur = decode_capitals(&cur);
    }
    cur
}

#[cfg(test)]
mod tests {
    use super::*;
    use capitals::{encode_capitals, decode_capitals, CAP_ESC};
    use markup::{encode_dict, decode_dict};

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
    fn encode_decode_cap_marker_bytes() {
        assert_roundtrip(&[0x01]);
        assert_roundtrip(&[0x01, b'a']);
        assert_roundtrip(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
        assert_roundtrip(&[CAP_ESC]);
        assert_roundtrip(&[CAP_ESC, CAP_ESC]);
        assert_roundtrip(&[0x00, 0x01, 0x7F, 0x08, b' ', b'A']);
        let mut all_ctrl = Vec::new();
        for b in 0x00u8..=0x1F {
            all_ctrl.push(b);
        }
        assert_roundtrip(&all_ctrl);
    }

    #[test]
    fn encode_decode_random_bytes() {
        let mut data = Vec::with_capacity(4096);
        let mut state: u32 = 0xC0FFEE;
        for _ in 0..4096 {
            state = state
                .wrapping_mul(1664525)
                .wrapping_add(1013904223);
            data.push((state >> 24) as u8);
        }
        assert_roundtrip(&data);
    }

    #[test]
    fn capitals_only_roundtrip() {
        let s = b"Alpha beta Gamma\nDelta >Epsilon \"Zeta\" 'Eta' [Theta] (Iota)";
        let e = encode_capitals(s);
        assert_eq!(decode_capitals(&e), s);
        let ctrl = [0x01u8, 0x08, 0x7F, b' ', b'A'];
        assert_eq!(decode_capitals(&encode_capitals(&ctrl)), ctrl);
    }

    #[test]
    fn dict_shrinks_common_tags() {
        let s = b"<page><title>X</title></page>";
        let e = encode_dict(s);
        assert!(e.len() < s.len());
        assert_eq!(decode_dict(&e), s);
    }
}
