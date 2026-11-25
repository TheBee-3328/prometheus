//! Word-context bit model for alphabetic runs. Integer only.
//!
//! DIAG-R6: StateMap dual-rate (p+n) replaces Laplace n0/n1 — hybrid without
//! a second map layer; same asymptotic count schedule as other StateMaps.

use crate::model::Model;
use crate::params::WORD_BITS;
use crate::statemap::{sm_init, sm_predict, sm_update};

/// Hashed whole-word context + bit path.
pub struct WordModel {
    /// Rolling hash of letters in the current word (0 if not in word).
    whash: u32,
    in_word: bool,
    bit_cxt: u16,
    cells: Box<[u32]>,
    mask: usize,
}

impl WordModel {
    pub fn new() -> Self {
        let size = 1usize << WORD_BITS;
        Self {
            whash: 0,
            in_word: false,
            bit_cxt: 1,
            cells: vec![sm_init(); size].into_boxed_slice(),
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
        sm_predict(self.cells[i])
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        if self.in_word || self.whash != 0 {
            let i = self.index();
            self.cells[i] = sm_update(self.cells[i], bit);
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
