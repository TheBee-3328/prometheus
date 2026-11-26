//! Word-context bit model for alphabetic runs. Integer only.

use crate::model::Model;
use crate::params::{ORDER2_RESCALE, WORD_BITS, WORD_CTX_MODE};

#[inline]
fn pack_counts(n0: u8, n1: u8) -> u16 {
    (n0 as u16) | ((n1 as u16) << 8)
}

#[inline]
fn unpack_counts(c: u16) -> (u8, u8) {
    (c as u8, (c >> 8) as u8)
}

/// Hashed whole-word context + bit path.
pub struct WordModel {
    /// Rolling hash of letters in the current word (0 if not in word).
    whash: u32,
    /// Hash of the previous completed word (0 if none).
    prev_hash: u32,
    /// 0 = outside word; else letter index capped at 31.
    wpos: u8,
    in_word: bool,
    bit_cxt: u16,
    /// Packed n0/n1 (lo8=n0, hi8=n1).
    counts: Box<[u16]>,
    /// 8-bit context fingerprint; 0 = empty.
    chk: Box<[u8]>,
    mask: usize,
    /// Cached slot from last `predict_hit` (valid when `have_slot`).
    last_i: usize,
    last_cs: u8,
    have_slot: bool,
}

impl WordModel {
    pub fn new() -> Self {
        let size = 1usize << WORD_BITS;
        Self {
            whash: 0,
            prev_hash: 0,
            wpos: 0,
            in_word: false,
            bit_cxt: 1,
            counts: vec![0u16; size].into_boxed_slice(),
            chk: vec![0u8; size].into_boxed_slice(),
            mask: size - 1,
            last_i: 0,
            last_cs: 0,
            have_slot: false,
        }
    }

    #[inline]
    fn index_cs(&self) -> (usize, u8) {
        let mut h = self.whash ^ ((self.bit_cxt as u32) << 16);
        h = h.wrapping_mul(16777619);
        // Optional richer NL contexts (compile-time WORD_CTX_MODE).
        match WORD_CTX_MODE {
            1 => {
                // Previous word hash.
                h ^= self.prev_hash.wrapping_mul(0x9e3779b9);
                h = h.wrapping_mul(16777619);
            }
            2 => {
                // Previous word + position in current word.
                h ^= self.prev_hash.wrapping_mul(0x9e3779b9);
                h = h.wrapping_mul(16777619);
                h ^= self.wpos as u32;
                h = h.wrapping_mul(16777619);
            }
            3 => {
                // Position only (no prev).
                h ^= self.wpos as u32;
                h = h.wrapping_mul(16777619);
            }
            4 => {
                // Suffix-ish: fold prev weakly + pos (lighter prev mix).
                h ^= self.prev_hash >> 1;
                h = h.wrapping_mul(16777619);
                h ^= (self.wpos as u32) << 4;
                h = h.wrapping_mul(16777619);
            }
            _ => {}
        }
        let idx = (h as usize) & self.mask;
        let mut cs = (h.wrapping_mul(0x85ebca6b) >> 24) as u8;
        if cs == 0 {
            cs = 1;
        }
        (idx, cs)
    }

    #[inline]
    fn is_letter(b: u8) -> bool {
        matches!(b, b'a'..=b'z' | b'A'..=b'Z')
    }

    #[inline]
    fn prob(n0: u8, n1: u8) -> u16 {
        let a = n1 as u32 + 1;
        let b = n0 as u32 + n1 as u32 + 2;
        let p = (a * 65535) / b;
        if p < 1 {
            1
        } else if p > 65534 {
            65534
        } else {
            p as u16
        }
    }
}

impl Default for WordModel {
    fn default() -> Self {
        Self::new()
    }
}

impl WordModel {
    /// Hit-gated prediction: `None` on inactive / checksum miss / cold slot.
    #[inline]
    pub fn predict_hit(&mut self) -> Option<u16> {
        if !self.in_word && self.whash == 0 {
            self.have_slot = false;
            return None;
        }
        let (i, cs) = self.index_cs();
        self.last_i = i;
        self.last_cs = cs;
        self.have_slot = true;
        if self.chk[i] != cs {
            return None;
        }
        let (n0, n1) = unpack_counts(self.counts[i]);
        if n0 == 0 && n1 == 0 {
            return None;
        }
        Some(Self::prob(n0, n1))
    }
}

impl Model for WordModel {
    #[inline]
    fn predict(&mut self) -> u16 {
        self.predict_hit().unwrap_or(32768)
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        if self.in_word || self.whash != 0 {
            let (i, cs) = if self.have_slot {
                (self.last_i, self.last_cs)
            } else {
                self.index_cs()
            };
            if self.chk[i] != cs {
                self.chk[i] = cs;
                self.counts[i] = 0;
            }
            let (mut n0, mut n1) = unpack_counts(self.counts[i]);
            if bit == 0 {
                n0 = n0.saturating_add(1);
            } else {
                n1 = n1.saturating_add(1);
            }
            if (n0 as u16) + (n1 as u16) >= ORDER2_RESCALE {
                n0 >>= 1;
                n1 >>= 1;
            }
            self.counts[i] = pack_counts(n0, n1);
        }
        self.have_slot = false;
        self.bit_cxt = (self.bit_cxt << 1) | (bit as u16);
        if self.bit_cxt >= 256 {
            let byte = (self.bit_cxt & 0xff) as u8;
            if Self::is_letter(byte) {
                let lc = if (b'A'..=b'Z').contains(&byte) {
                    byte + 32
                } else {
                    byte
                };
                self.whash = self.whash.wrapping_mul(263).wrapping_add(lc as u32);
                self.in_word = true;
                if self.wpos < 31 {
                    self.wpos = self.wpos.saturating_add(1);
                }
            } else {
                if self.in_word && self.whash != 0 {
                    self.prev_hash = self.whash;
                }
                self.whash = 0;
                self.in_word = false;
                self.wpos = 0;
            }
            self.bit_cxt = 1;
        }
    }
}
