//! Secondary symbol estimation (SSE / light APM). Integer only.
//!
//! Refines a primary probability using (bit_cxt, quantized p) counts.

use crate::params::ORDER2_RESCALE;

/// bit_cxt (8) × match_bucket (8) = 2048 contexts
const N_CXT: usize = 2048;
const N_BUCKET: usize = 64; // p >> 10
const SIZE: usize = N_CXT * N_BUCKET;

/// Adaptive SSE table: mixes primary p with a learned conditional.
pub struct Sse {
    n0: Box<[u8]>,
    n1: Box<[u8]>,
    /// last index for update
    last_i: usize,
    /// blend of SSE vs primary (0..=255)
    mix_w: u32,
}

impl Sse {
    pub fn new(mix_w: u32) -> Self {
        Self {
            n0: vec![0u8; SIZE].into_boxed_slice(),
            n1: vec![0u8; SIZE].into_boxed_slice(),
            last_i: 0,
            mix_w,
        }
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

    /// Match length → 0..7 bucket for SSE context.
    #[inline]
    pub fn match_bucket(match_len: u32) -> u16 {
        match match_len {
            0..=4 => 0,
            5..=6 => 1,
            7..=10 => 2,
            11..=20 => 3,
            21..=40 => 4,
            41..=80 => 5,
            81..=160 => 6,
            _ => 7,
        }
    }

    /// Refine `p` using `bit_cxt` and match-length bucket. Stores index for update.
    #[inline]
    pub fn refine(&mut self, p: u16, bit_cxt: u16, match_len: u32) -> u16 {
        let mb = Self::match_bucket(match_len) as usize;
        let cxt = ((bit_cxt as usize) & 255) | (mb << 8);
        let bucket = (p as usize) >> 10; // 0..63
        let bucket = bucket.min(N_BUCKET - 1);
        let i = cxt * N_BUCKET + bucket;
        self.last_i = i;
        let ps = Self::prob(self.n0[i], self.n1[i]);
        let w = self.mix_w;
        let out = (p as u32 * (256 - w) + ps as u32 * w) / 256;
        if out < 1 {
            1
        } else if out > 65534 {
            65534
        } else {
            out as u16
        }
    }

    #[inline]
    pub fn update(&mut self, bit: u8) {
        let i = self.last_i;
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
}
