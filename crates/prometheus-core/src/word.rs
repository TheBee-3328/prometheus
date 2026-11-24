//! Word-context bit model for alphabetic runs. Integer only.

use crate::model::Model;
use crate::params::{ORDER2_RESCALE, WORD_BITS};

/// Hashed whole-word context + bit path.
pub struct WordModel {
    /// Rolling hash of letters in the current word (0 if not in word).
    whash: u32,
    in_word: bool,
    bit_cxt: u16,
    n0: Box<[u8]>,
    n1: Box<[u8]>,
    mask: usize,
}

impl WordModel {
    pub fn new() -> Self {
        let size = 1usize << WORD_BITS;
        Self {
            whash: 0,
            in_word: false,
            bit_cxt: 1,
            n0: vec![0u8; size].into_boxed_slice(),
            n1: vec![0u8; size].into_boxed_slice(),
            mask: size - 1,
        }
    }

    #[inline]
    fn index(&self) -> usize {
        let mut h = self.whash ^ ((self.bit_cxt as u32) << 16);
        h = h.wrapping_mul(16777619);
        (h as usize) & self.mask
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

impl Model for WordModel {
    #[inline]
    fn predict(&mut self) -> u16 {
        if !self.in_word && self.whash == 0 {
            return 32768;
        }
        let i = self.index();
        Self::prob(self.n0[i], self.n1[i])
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        if self.in_word || self.whash != 0 {
            let i = self.index();
            if bit == 0 {
                self.n0[i] = self.n0[i].saturating_add(1);
            } else {
                self.n1[i] = self.n1[i].saturating_add(1);
            }
            if (self.n0[i] as u16) + (self.n1[i] as u16) >= ORDER2_RESCALE {
                self.n0[i] >>= 1;
                self.n1[i] >>= 1;
            }
        }
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
            } else {
                self.whash = 0;
                self.in_word = false;
            }
            self.bit_cxt = 1;
        }
    }
}
