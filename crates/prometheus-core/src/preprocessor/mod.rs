//! Reversible preprocessor for enwik-style XML/text.
//!
//! Encode order (decode reverses):
//! 1. Capitals fold (optional)
//! 2. Static markup dictionary + mined templates (optional, one greedy pass)
//! 3. Natural-language word/phrase dictionary (optional)
//! 4. Side-stream pack for template params when `!PARAM_INLINE`
//!
//! Cap markers occupy `0x01..=0x08`. Literal bytes in that range (and the
//! escape byte `0x7F`) are prefixed with `CAP_ESC` so encode/decode are
//! inverses for every `&[u8]`.

pub mod capitals;
pub mod markup;
pub mod stages;
pub mod templates;
pub mod words;

use capitals::{decode_capitals, encode_capitals};
use stages::{PARAM_INLINE, PRE_STAGE_CAPITALS, PRE_STAGE_MARKUP, PRE_STAGE_TEMPLATES, PRE_STAGE_WORDS};
use templates::{
    decode_markup_templates, encode_markup_templates, pack_side_stream, unpack_side_stream,
    uses_side_stream,
};
use words::{decode_words, encode_words};

/// Encode: capitals → markup+templates → words → optional side-stream pack.
pub fn encode(input: &[u8]) -> Vec<u8> {
    let mut cur = input.to_vec();
    if PRE_STAGE_CAPITALS {
        cur = encode_capitals(&cur);
    }
    let mut side = Vec::new();
    if PRE_STAGE_MARKUP || PRE_STAGE_TEMPLATES {
        let (body, s) = encode_markup_templates(&cur);
        cur = body;
        side = s;
    }
    if PRE_STAGE_WORDS {
        cur = encode_words(&cur);
    }
    if uses_side_stream() && !PARAM_INLINE {
        pack_side_stream(&cur, &side)
    } else {
        // Inline (or no templates): side must be empty.
        debug_assert!(side.is_empty() || PARAM_INLINE);
        cur
    }
}

/// Decode: optional side-stream unpack → words → markup+templates → capitals.
pub fn decode(input: &[u8]) -> Vec<u8> {
    let (mut cur, side) = if uses_side_stream() {
        unpack_side_stream(input)
    } else {
        (input.to_vec(), Vec::new())
    };
    if PRE_STAGE_WORDS {
        cur = decode_words(&cur);
    }
    if PRE_STAGE_MARKUP || PRE_STAGE_TEMPLATES {
        cur = decode_markup_templates(&cur, &side);
    }
    if PRE_STAGE_CAPITALS {
        cur = decode_capitals(&cur);
    }
    cur
}

#[cfg(test)]
mod tests {
    use super::*;
    use capitals::{decode_capitals, encode_capitals, CAP_ESC};
    use markup::{decode_dict, encode_dict};

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

    #[test]
    fn templateish_roundtrip() {
        assert_roundtrip(b"{{ref}}");
        assert_roundtrip(b"{{cite web|url=http://example.com|title=Hello}}");
        assert_roundtrip(b"pre {{note|abc}} mid {{flagicon|USA}} post");
        assert_roundtrip(b"{{nested|a={{inner|x}}|b=y}}");
    }

    #[test]
    fn succession_sibling_does_not_span() {
        // Two succession-like boxes: matcher must not join them across `}}`.
        let s = b"{{succession box|title=A|x|before=B|after=C|years=1}}\n{{succession box|title=D|before=E|after=F|years=2}}";
        assert_roundtrip(s);
        let e = encode(s);
        let d = decode(&e);
        assert_eq!(d, s);
    }
}
