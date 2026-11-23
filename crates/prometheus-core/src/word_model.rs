//! Whole-word / after-space context model (PAQ-style) for English/XML text.
//! Integer / fixed-point only; no FP, no HashMap.

use crate::model::Model;

/// Hash-table size: 2^22 entries (~4M). Two tables → ~16 MiB counters.
const TABLE_BITS: usize = 22;
const TABLE: usize = 1 << TABLE_BITS;
const TABLE_MASK: u32 = (TABLE as u32) - 1;

/// Word unigram + bigram bit contexts with intra-byte bit path (MSB first).
///
/// A "word" is a run of letters (a–z folded, or bytes ≥128). On a non-letter
/// the completed word hash shifts into `word1`/`word2`. Contexts also cover
/// the start of a word after space/newline via empty `word0` + prior word.
pub struct WordModel {
    word0: u32,
    word1: u32,
    word2: u32,
    prev: u8,
    bit_cxt: u16,
    /// Unigram: hash(word0, prev, bit_cxt)
    n0_u: Box<[u8]>,
    n1_u: Box<[u8]>,
    /// Bigram: hash(word0, word1, bit_cxt)
    n0_b: Box<[u8]>,
    n1_b: Box<[u8]>,
}

impl WordModel {
    pub fn new() -> Self {
        Self {
            word0: 0,
            word1: 0,
            word2: 0,
            prev: 0,
            bit_cxt: 1,
            n0_u: vec![0u8; TABLE].into_boxed_slice(),
            n1_u: vec![0u8; TABLE].into_boxed_slice(),
            n0_b: vec![0u8; TABLE].into_boxed_slice(),
            n1_b: vec![0u8; TABLE].into_boxed_slice(),
        }
    }

    #[inline]
    fn hash2(a: u32, b: u32) -> usize {
        (a.wrapping_mul(0x9E37_79B1)
            .wrapping_add(b.wrapping_mul(0x85EB_CA6B))
            .wrapping_add(0xC2B2_AE35)
            & TABLE_MASK) as usize
    }

    #[inline]
    fn hash3(a: u32, b: u32, c: u32) -> usize {
        (a.wrapping_mul(0x9E37_79B1)
            .wrapping_add(b.wrapping_mul(0x85EB_CA6B))
            .wrapping_add(c.wrapping_mul(0xC2B2_AE35))
            & TABLE_MASK) as usize
    }

    #[inline]
    fn uni_index(&self) -> usize {
        Self::hash3(self.word0, self.prev as u32, self.bit_cxt as u32)
    }

    #[inline]
    fn bi_index(&self) -> usize {
        // Include word2 lightly so after-space (word0==0) still sees history.
        let w = self.word1.wrapping_add(self.word2.wrapping_mul(3));
        Self::hash2(
            self.word0.wrapping_mul(271).wrapping_add(w),
            self.bit_cxt as u32,
        )
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

    /// True while assembling a letter-run (partial word hash non-zero).
    #[inline]
    pub fn in_word(&self) -> bool {
        self.word0 != 0
    }

    /// Prediction and total count at current context (for external mixing).
    #[inline]
    pub fn predict_counts(&self) -> (u16, u16) {
        let iu = self.uni_index();
        let ib = self.bi_index();
        let n0u = self.n0_u[iu];
        let n1u = self.n1_u[iu];
        let n0b = self.n0_b[ib];
        let n1b = self.n1_b[ib];
        let pu = Self::prob(n0u, n1u);
        let pb = Self::prob(n0b, n1b);
        let wu = (n0u as u32) + (n1u as u32) + 1;
        let wb = (n0b as u32) + (n1b as u32) + 1;
        let p = (pu as u32 * wu + pb as u32 * wb) / (wu + wb);
        let p = if p < 1 {
            1
        } else if p > 65534 {
            65534
        } else {
            p as u16
        };
        let n = ((n0u as u16) + (n1u as u16) + (n0b as u16) + (n1b as u16)).min(65535);
        (p, n)
    }

    #[inline]
    fn bump(n0: &mut u8, n1: &mut u8, bit: u8) {
        if bit == 0 {
            *n0 = n0.saturating_add(1);
        } else {
            *n1 = n1.saturating_add(1);
        }
        if (*n0 as u16) + (*n1 as u16) >= 250 {
            *n0 >>= 1;
            *n1 >>= 1;
        }
    }

    #[inline]
    fn on_byte(&mut self, c: u8) {
        let folded = if (b'A'..=b'Z').contains(&c) {
            c + (b'a' - b'A')
        } else {
            c
        };
        // Letters + high bytes (UTF-8 / wiki markup tails) form words.
        if (b'a'..=b'z').contains(&folded) || c >= 128 {
            self.word0 = self
                .word0
                .wrapping_mul(263)
                .wrapping_mul(32)
                .wrapping_add(folded as u32);
        } else if self.word0 != 0 {
            self.word2 = self.word1.wrapping_mul(29);
            self.word1 = self.word0.wrapping_mul(11);
            self.word0 = 0;
        }
        // Digits: light trail when not inside a letter-word.
        if (b'0'..=b'9').contains(&c) && self.word0 == 0 {
            self.word2 = self.word2.wrapping_mul(10).wrapping_add((c - b'0') as u32);
        }
        self.prev = c;
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
        self.predict_counts().0
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        let iu = self.uni_index();
        let ib = self.bi_index();
        Self::bump(&mut self.n0_u[iu], &mut self.n1_u[iu], bit);
        Self::bump(&mut self.n0_b[ib], &mut self.n1_b[ib], bit);

        self.bit_cxt = (self.bit_cxt << 1) | (bit as u16);
        if self.bit_cxt >= 256 {
            let byte = (self.bit_cxt & 0xff) as u8;
            self.on_byte(byte);
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
    fn word_predict_update_lockstep() {
        let bits = bits_of(b"The quick brown fox </a> wiki");
        let mut a = WordModel::new();
        let mut b = WordModel::new();
        let mut preds = Vec::new();
        for &bit in &bits {
            preds.push(a.predict());
            a.update(bit);
        }
        for (i, &bit) in bits.iter().enumerate() {
            let p = b.predict();
            assert_eq!(p, preds[i], "predict mismatch at bit {i}");
            b.update(bit);
        }
    }

    #[test]
    fn word_learns_repeated_phrase() {
        let mut m = WordModel::new();
        let train = b" the cat the dog the car the ";
        let mut last_p = 32768u16;
        for &byte in train.iter().cycle().take(train.len() * 40) {
            for i in (0..8).rev() {
                let bit = (byte >> i) & 1;
                last_p = m.predict();
                m.update(bit);
            }
        }
        let (_p, n) = m.predict_counts();
        assert!(n > 0, "expected non-zero counts after training, n={n}");
        // Should have moved off a fresh Laplace midpoint somewhere in the stream.
        assert_ne!(last_p, 32768, "expected adapted probability, got {last_p}");
    }
}
