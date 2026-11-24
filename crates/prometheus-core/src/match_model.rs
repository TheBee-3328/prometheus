//! LZ-style longest-match context model.
//!
//! Hash of the last few bytes maps to a prior buffer position. When the
//! context verifies, subsequent bits are predicted from the byte that followed
//! that prior occurrence. Mixed with order-2 only for sufficiently long matches.

use crate::apm::{p12_to_p16, p16_to_p12, Apm};
use crate::indirect::IndirectModel;
use crate::model::{HashOrderModel, Model, Order1Model, Order2Model};
use crate::order0::Order0Model;
use crate::params::{
    match_conf, mix_weight, APM_MIX_WEIGHT, APM_RATE_SHIFT, BUF_MASK, BUF_SIZE, HASH_MASK,
    HASH_ORDER, HASH_SIZE, INDIRECT_MIX_WEIGHT, MATCH_IND_CROSS_WEIGHT, MIN_MIX_LEN,
    ORDER0_MIX_WEIGHT, ORDER1_MIX_WEIGHT, ORDER3_BITS, ORDER3_MIX_WEIGHT, ORDER4_BITS,
    ORDER4_MIX_WEIGHT, ORDER5_BITS, ORDER5_MIX_WEIGHT, ORDER6_BITS, ORDER6_MIX_WEIGHT,
    ORDER7_BITS, ORDER7_MIX_WEIGHT, SSE2_MIX_WEIGHT, SSE_MIX_WEIGHT, STRUCT_MIX_WEIGHT,
    WORD_MIX_WEIGHT,
};
use crate::statemap::{sm_init, sm_predict, sm_update};
use crate::sparse::SparseModel;
use crate::sse::{Sse, Sse2};
use crate::word::WordModel;

/// Number of hash-table candidate slots checked when starting a match.
/// Slot 0 = primary (with fingerprint); slots 1.. = prior occupants (chain).
const MATCH_CANDIDATES: usize = 4;

/// LZ match model: buffer + hash → expected next bits.
pub struct MatchModel {
    buf: Box<[u8]>,
    /// Bytes written so far (also next write index into `buf` via mask).
    pos: usize,
    ht: Box<[u32]>,
    /// 16-bit fingerprint of context hash (collision reject) for primary slot.
    ht_fp: Box<[u16]>,
    /// Prior occupants of each hash slot (depth MATCH_CANDIDATES-1).
    /// Indexed as `ht_prev[slot * HASH_SIZE + h]`.
    ht_prev: Box<[u32]>,
    /// Absolute index of next expected matched byte.
    match_pos: usize,
    /// Matched bytes so far (0 = inactive).
    match_len: u32,
    expected: u8,
    bit_cxt: u16,
}

impl MatchModel {
    pub fn new() -> Self {
        Self {
            buf: vec![0u8; BUF_SIZE].into_boxed_slice(),
            pos: 0,
            ht: vec![0u32; HASH_SIZE].into_boxed_slice(),
            ht_fp: vec![0u16; HASH_SIZE].into_boxed_slice(),
            ht_prev: vec![0u32; HASH_SIZE * (MATCH_CANDIDATES - 1)].into_boxed_slice(),
            match_pos: 0,
            match_len: 0,
            expected: 0,
            bit_cxt: 1,
        }
    }

    #[inline]
    pub fn match_len(&self) -> u32 {
        self.match_len
    }

    /// Expected next bit under an active match (0 if inactive).
    #[inline]
    pub fn expected_bit_pub(&self) -> u8 {
        if self.match_len == 0 {
            0
        } else {
            self.expected_bit()
        }
    }

    /// Bits completed in the current byte = floor(log2(bit_cxt)).
    #[inline]
    fn bits_seen(bit_cxt: u16) -> u8 {
        (15u32.saturating_sub(bit_cxt.leading_zeros() as u32)) as u8
    }

    #[inline]
    fn expected_bit(&self) -> u8 {
        let bit_index = Self::bits_seen(self.bit_cxt);
        (self.expected >> (7 - bit_index)) & 1
    }

    #[inline]
    fn context_hash_raw(&self, end: usize) -> u32 {
        // FNV-ish hash of buf[end-HASH_ORDER+1 ..= end]
        let mut h: u32 = 2166136261;
        for i in (0..HASH_ORDER).rev() {
            let b = self.buf[end.wrapping_sub(i) & BUF_MASK] as u32;
            h ^= b;
            h = h.wrapping_mul(16777619);
        }
        h
    }

    #[inline]
    fn context_hash(&self, end: usize) -> (usize, u16) {
        let h = self.context_hash_raw(end);
        ((h as usize) & HASH_MASK, (h >> 16) as u16)
    }

    #[inline]
    fn predict_from_match(&self) -> u16 {
        if self.match_len < MIN_MIX_LEN {
            return 32768;
        }
        let bit_index = Self::bits_seen(self.bit_cxt);
        let expected_bit = (self.expected >> (7 - bit_index)) & 1;
        let conf = match_conf(self.match_len);
        let p = if expected_bit == 1 {
            32768u32 + conf
        } else {
            32768u32.saturating_sub(conf)
        };
        if p < 1 {
            1
        } else if p > 65534 {
            65534
        } else {
            p as u16
        }
    }

    fn finish_byte(&mut self, byte: u8) {
        let p = self.pos;
        self.buf[p & BUF_MASK] = byte;

        // Extend an active match after a fully matching byte.
        if self.match_len > 0 {
            self.match_len = self.match_len.saturating_add(1);
            self.match_pos = self.match_pos.wrapping_add(1);
            // Stop before reading the byte we just wrote as "future".
            if self.match_pos >= p + 1 || self.match_len > 0xffff {
                self.match_len = 0;
            } else {
                self.expected = self.buf[self.match_pos & BUF_MASK];
            }
        }

        // Index context ending at `p`; look up prior occurrence(s).
        if p + 1 >= HASH_ORDER {
            let (h, fp) = self.context_hash(p);
            let primary = self.ht[h];
            let primary_fp = self.ht_fp[h];

            // Snapshot candidates before mutating the chain.
            if self.match_len == 0 {
                let mut best_len = 0u32;
                let mut best_pos = 0usize;

                // Candidate 0: primary with fingerprint reject.
                if primary > 0 && (primary as usize) < p && primary_fp == fp {
                    let (ctx, next) = self.verify_match(p, primary as usize);
                    if ctx >= HASH_ORDER as u32 && next <= p && ctx > best_len {
                        best_len = ctx;
                        best_pos = next;
                    }
                }
                // Candidates 1..: prior occupants (byte-verify only).
                for slot in 0..(MATCH_CANDIDATES - 1) {
                    let cand = self.ht_prev[slot * HASH_SIZE + h] as usize;
                    if cand > 0 && cand < p {
                        let (ctx, next) = self.verify_match(p, cand);
                        if ctx >= HASH_ORDER as u32 && next <= p && ctx > best_len {
                            best_len = ctx;
                            best_pos = next;
                        }
                    }
                }

                if best_len >= HASH_ORDER as u32 {
                    self.match_pos = best_pos;
                    self.match_len = best_len;
                    self.expected = self.buf[best_pos & BUF_MASK];
                }
            }

            // Push primary into the prev chain, then store current.
            if MATCH_CANDIDATES > 2 {
                for slot in (1..MATCH_CANDIDATES - 1).rev() {
                    self.ht_prev[slot * HASH_SIZE + h] =
                        self.ht_prev[(slot - 1) * HASH_SIZE + h];
                }
            }
            if MATCH_CANDIDATES > 1 {
                self.ht_prev[h] = primary;
            }
            self.ht[h] = p as u32;
            self.ht_fp[h] = fp;
        }

        self.pos = p + 1;
        self.bit_cxt = 1;
    }

    /// Verify how many trailing bytes are shared between `p` and `candidate`.
    /// Returns (match_len, next_expected_pos).
    #[inline]
    fn verify_match(&self, p: usize, candidate: usize) -> (u32, usize) {
        let mut ctx = 0usize;
        let max = (p - candidate).min(255);
        while ctx < max {
            if self.buf[p.wrapping_sub(ctx) & BUF_MASK]
                != self.buf[candidate.wrapping_sub(ctx) & BUF_MASK]
            {
                break;
            }
            ctx += 1;
        }
        (ctx as u32, candidate + 1)
    }
}

impl Default for MatchModel {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for MatchModel {
    #[inline]
    fn predict(&mut self) -> u16 {
        self.predict_from_match()
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        if self.match_len > 0 && bit != self.expected_bit() {
            self.match_len = 0;
        }

        self.bit_cxt = (self.bit_cxt << 1) | (bit as u16);
        if self.bit_cxt >= 256 {
            let byte = (self.bit_cxt & 0xff) as u8;
            self.finish_byte(byte);
        }
    }
}

/// Match×indirect cross: StateMap over match_bucket × ind1 × bit_pos × expected_bit.
const CROSS_SIZE: usize = 8 * 256 * 8 * 2; // 32768

/// Mix order-1/2/3/4 + sparse/XML with the match model; match weight grows with length.
/// Sparse/structure only blend in when match is inactive (do not fight match).
pub struct Order2MatchModel {
    order0: Order0Model,
    order1: Order1Model,
    order2: Order2Model,
    order3: HashOrderModel,
    order4: HashOrderModel,
    order5: HashOrderModel,
    order6: HashOrderModel,
    order7: HashOrderModel,
    match_m: MatchModel,
    sparse: SparseModel,
    word: WordModel,
    indirect: IndirectModel,
    /// Match×indirect cross StateMap cells.
    cross: Box<[u32]>,
    cross_i: usize,
    sse: Sse,
    sse2: Sse2,
    /// Third-stage paq-style APM (order-1 byte × match bucket).
    apm: Apm,
    /// Shared bit path for SSE context (mirrors order2 bit_cxt).
    bit_cxt: u16,
    last_class: u8,
    last_byte: u8,
}

impl Order2MatchModel {
    pub fn new() -> Self {
        Self {
            order0: Order0Model::new(),
            order1: Order1Model::new(),
            order2: Order2Model::new(),
            order3: HashOrderModel::new(3, ORDER3_BITS),
            order4: HashOrderModel::new(4, ORDER4_BITS),
            order5: HashOrderModel::new(5, ORDER5_BITS),
            order6: HashOrderModel::new(6, ORDER6_BITS),
            order7: HashOrderModel::new(7, ORDER7_BITS),
            match_m: MatchModel::new(),
            sparse: SparseModel::new(),
            word: WordModel::new(),
            indirect: IndirectModel::new(),
            cross: vec![sm_init(); CROSS_SIZE].into_boxed_slice(),
            cross_i: 0,
            sse: Sse::new(SSE_MIX_WEIGHT),
            sse2: Sse2::new(SSE2_MIX_WEIGHT),
            // 256 last-byte × 8 bit_pos × 8 match buckets = 16384 contexts.
            apm: Apm::new(16384).with_rate_shift(APM_RATE_SHIFT),
            bit_cxt: 1,
            last_class: 8,
            last_byte: 0,
        }
    }

    #[inline]
    fn bits_seen(bit_cxt: u16) -> u8 {
        (15u32.saturating_sub(bit_cxt.leading_zeros() as u32)) as u8
    }

    #[inline]
    fn byte_class(b: u8) -> u8 {
        match b {
            b' ' | b'\n' | b'\t' | b'\r' => 0,
            b'a'..=b'z' | b'A'..=b'Z' => 1,
            b'0'..=b'9' => 2,
            b'<' => 3,
            b'>' => 4,
            b'&' | b';' => 5,
            b'[' | b']' | b'{' | b'}' => 6,
            0x80..=0xfe => 7,
            _ => 8,
        }
    }

    #[inline]
    fn mix_w(p_a: u16, p_b: u16, w: u32) -> u16 {
        let p = (p_a as u32 * (256 - w) + p_b as u32 * w) / 256;
        if p < 1 {
            1
        } else if p > 65534 {
            65534
        } else {
            p as u16
        }
    }

    #[inline]
    fn mix_match(p_o2: u16, p_m: u16, match_len: u32) -> u16 {
        Self::mix_w(p_o2, p_m, mix_weight(match_len))
    }
}

impl Default for Order2MatchModel {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for Order2MatchModel {
    #[inline]
    fn predict(&mut self) -> u16 {
        let p0u = self.order0.predict();
        let p1 = self.order1.predict();
        let p2 = self.order2.predict();
        let p3 = self.order3.predict();
        let p4 = self.order4.predict();
        let p5 = self.order5.predict();
        let p6 = self.order6.predict();
        let p7 = self.order7.predict();
        let mut p0 = Self::mix_w(p2, p1, ORDER1_MIX_WEIGHT);
        p0 = Self::mix_w(p0, p3, ORDER3_MIX_WEIGHT);
        p0 = Self::mix_w(p0, p4, ORDER4_MIX_WEIGHT);
        p0 = Self::mix_w(p0, p5, ORDER5_MIX_WEIGHT);
        p0 = Self::mix_w(p0, p6, ORDER6_MIX_WEIGHT);
        p0 = Self::mix_w(p0, p7, ORDER7_MIX_WEIGHT);
        p0 = Self::mix_w(p0, p0u, ORDER0_MIX_WEIGHT);
        let pw = self.word.predict();
        p0 = Self::mix_w(p0, pw, WORD_MIX_WEIGHT);
        let pi = self.indirect.predict();
        p0 = Self::mix_w(p0, pi, INDIRECT_MIX_WEIGHT);
        let len = self.match_m.match_len();
        // Match×indirect cross: mb × ind1 × bit_pos × expected_bit.
        let mb = Sse::match_bucket(len) as usize;
        let bp = (Self::bits_seen(self.bit_cxt) as usize) & 7;
        let eb = self.match_m.expected_bit_pub() as usize & 1;
        let ind1 = self.indirect.ind1() as usize;
        self.cross_i = (((mb << 8) | ind1) << 3 | bp) << 1 | eb;
        let pc = sm_predict(self.cross[self.cross_i]);
        p0 = Self::mix_w(p0, pc, MATCH_IND_CROSS_WEIGHT);
        let p = if len >= MIN_MIX_LEN {
            let pm = self.match_m.predict();
            Self::mix_match(p0, pm, len)
        } else {
            let px = self.sparse.predict_struct();
            Self::mix_w(p0, px, STRUCT_MIX_WEIGHT)
        };
        let p = self.sse.refine(p, self.bit_cxt, len);
        let p = self.sse2.refine(p, self.bit_cxt, self.last_class);
        // APM: last_byte × bit_pos × match_bucket (richer than SSE bit_cxt alone).
        let cxt = ((self.last_byte as usize) << 6) | (bp << 3) | mb;
        let p12 = self.apm.predict(p16_to_p12(p), cxt);
        Self::mix_w(p, p12_to_p16(p12), APM_MIX_WEIGHT)
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        self.cross[self.cross_i] = sm_update(self.cross[self.cross_i], bit);
        self.apm.update(bit);
        self.sse.update(bit);
        self.sse2.update(bit);
        self.order0.update(bit);
        self.order1.update(bit);
        self.order2.update(bit);
        self.order3.update(bit);
        self.order4.update(bit);
        self.order5.update(bit);
        self.order6.update(bit);
        self.order7.update(bit);
        self.word.update(bit);
        self.indirect.update(bit);
        self.match_m.update(bit);
        self.sparse.update(bit);
        self.bit_cxt = (self.bit_cxt << 1) | (bit as u16);
        if self.bit_cxt >= 256 {
            let byte = (self.bit_cxt & 0xff) as u8;
            self.last_class = Self::byte_class(byte);
            self.last_byte = byte;
            self.bit_cxt = 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bits_of(bytes: &[u8]) -> Vec<u8> {
        bytes
            .iter()
            .flat_map(|b| (0..8).rev().map(move |i| (b >> i) & 1))
            .collect()
    }

    #[test]
    fn match_predict_update_lockstep() {
        let data = b"abcabracadabra abracadabra abracadabra!";
        let bits = bits_of(data);
        let mut a = MatchModel::new();
        let mut b = MatchModel::new();
        let mut preds = Vec::new();
        for &bit in &bits {
            preds.push(a.predict());
            a.update(bit);
        }
        for (i, &bit) in bits.iter().enumerate() {
            assert_eq!(b.predict(), preds[i], "predict mismatch at bit {i}");
            b.update(bit);
        }
    }

    #[test]
    fn match_finds_repetition() {
        let mut block = Vec::new();
        let phrase = b"The quick brown fox jumps over the lazy dog. ";
        for _ in 0..40 {
            block.extend_from_slice(phrase);
        }
        let bits = bits_of(&block);
        let mut m = MatchModel::new();
        let mut max_len = 0u32;
        let mut strong = 0u32;
        for &bit in &bits {
            let p = m.predict();
            if m.match_len() >= MIN_MIX_LEN {
                max_len = max_len.max(m.match_len());
                if !(30000..=35000).contains(&p) {
                    strong += 1;
                }
            }
            m.update(bit);
        }
        assert!(max_len >= 8, "expected long match, got max_len={max_len}");
        assert!(strong > 100, "expected many confident predictions, got {strong}");
    }

    #[test]
    fn order2_match_lockstep() {
        let data = b"<title>Wikipedia</title><text>repeat repeat repeat</text>";
        let bits = bits_of(data);
        let mut a = Order2MatchModel::new();
        let mut b = Order2MatchModel::new();
        let mut preds = Vec::new();
        for &bit in &bits {
            preds.push(a.predict());
            a.update(bit);
        }
        for (i, &bit) in bits.iter().enumerate() {
            assert_eq!(b.predict(), preds[i], "ensemble mismatch at bit {i}");
            b.update(bit);
        }
    }

    #[test]
    fn order2_match_lockstep_short_no_match_window() {
        // With order-1 in the ensemble, predictions differ from pure order-2;
        // still require lockstep between two ensemble instances on short input.
        let mut a = Order2MatchModel::new();
        let mut b = Order2MatchModel::new();
        let bits = bits_of(b"xyz");
        for &bit in &bits {
            assert_eq!(a.predict(), b.predict());
            a.update(bit);
            b.update(bit);
        }
    }

    #[test]
    fn order2_match_beats_order2_on_repeats() {
        use crate::arithmetic::{ArithmeticDecoder, ArithmeticEncoder};
        use std::io::Cursor;

        let mut data = Vec::new();
        let phrase = b"<mediawiki><page><title>Foo</title><text>bar baz qux</text></page>";
        for _ in 0..200 {
            data.extend_from_slice(phrase);
        }

        fn compressed_size<M: Model>(mut model: M, data: &[u8]) -> usize {
            let mut enc = ArithmeticEncoder::new(Cursor::new(Vec::new()));
            for &byte in data {
                for i in (0..8).rev() {
                    let bit = (byte >> i) & 1;
                    let p = model.predict();
                    enc.encode_bit(bit, p).unwrap();
                    model.update(bit);
                }
            }
            enc.finish().unwrap().into_inner().len()
        }

        let s_o2 = compressed_size(Order2Model::new(), &data);
        let s_m = compressed_size(Order2MatchModel::new(), &data);
        assert!(
            s_m < s_o2,
            "match ensemble should beat order-2 on repeats: match={s_m} o2={s_o2}"
        );

        // Roundtrip sanity with ensemble.
        let mut model = Order2MatchModel::new();
        let mut enc = ArithmeticEncoder::new(Cursor::new(Vec::new()));
        for &byte in &data {
            for i in (0..8).rev() {
                let bit = (byte >> i) & 1;
                let p = model.predict();
                enc.encode_bit(bit, p).unwrap();
                model.update(bit);
            }
        }
        let blob = enc.finish().unwrap().into_inner();
        let mut model = Order2MatchModel::new();
        let mut dec = ArithmeticDecoder::new(Cursor::new(blob)).unwrap();
        let mut out = Vec::with_capacity(data.len());
        for _ in 0..data.len() {
            let mut byte = 0u8;
            for _ in 0..8 {
                let p = model.predict();
                let bit = dec.decode_bit(p).unwrap();
                model.update(bit);
                byte = (byte << 1) | bit;
            }
            out.push(byte);
        }
        assert_eq!(out, data);
    }
}
