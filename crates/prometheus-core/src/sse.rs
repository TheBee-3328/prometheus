//! Secondary symbol estimation (SSE / light APM). Integer only.
//!
//! Refines a primary probability using (bit_cxt, quantized p) counts.
//! Counts are packed as `u16 = n0 | (n1 << 8)` so each slot is one cache line touch.

use crate::params::ORDER2_RESCALE;

/// bit_cxt (8) × match_bucket (8) = 2048 contexts
const N_CXT: usize = 2048;
const N_BUCKET: usize = 64; // p >> 10
const SIZE: usize = N_CXT * N_BUCKET;

#[inline]
fn pack_counts(n0: u8, n1: u8) -> u16 {
    (n0 as u16) | ((n1 as u16) << 8)
}

#[inline]
fn unpack_counts(c: u16) -> (u8, u8) {
    (c as u8, (c >> 8) as u8)
}

#[inline]
fn laplace_prob(n0: u8, n1: u8) -> u16 {
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

#[inline]
fn learn_packed(counts: &mut [u16], i: usize, bit: u8) {
    let (mut n0, mut n1) = unpack_counts(counts[i]);
    if bit == 0 {
        n0 = n0.saturating_add(1);
    } else {
        n1 = n1.saturating_add(1);
    }
    if (n0 as u16) + (n1 as u16) >= ORDER2_RESCALE {
        n0 >>= 1;
        n1 >>= 1;
    }
    counts[i] = pack_counts(n0, n1);
}

/// Adaptive SSE table: mixes primary p with a learned conditional.
pub struct Sse {
    /// Packed n0/n1 per slot (lo8=n0, hi8=n1).
    counts: Box<[u16]>,
    /// last index for update
    last_i: usize,
    /// blend of SSE vs primary (0..=255)
    mix_w: u32,
}

impl Sse {
    pub fn new(mix_w: u32) -> Self {
        Self {
            counts: vec![0u16; SIZE].into_boxed_slice(),
            last_i: 0,
            mix_w,
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
        let (n0, n1) = unpack_counts(self.counts[i]);
        let ps = laplace_prob(n0, n1);
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
        learn_packed(&mut self.counts, self.last_i, bit);
    }
}

/// Second-stage SSE: contexts from last byte class × bit_cxt × coarse p.
const N2_CLASS: usize = 9;
const N2_BIT: usize = 256;
const N2_BUCKET: usize = 32; // p >> 11
const SIZE2: usize = N2_CLASS * N2_BIT * N2_BUCKET;

pub struct Sse2 {
    counts: Box<[u16]>,
    last_i: usize,
    mix_w: u32,
}

impl Sse2 {
    pub fn new(mix_w: u32) -> Self {
        Self {
            counts: vec![0u16; SIZE2].into_boxed_slice(),
            last_i: 0,
            mix_w,
        }
    }

    #[inline]
    pub fn refine(&mut self, p: u16, bit_cxt: u16, last_class: u8) -> u16 {
        let c = (last_class as usize).min(N2_CLASS - 1);
        let bxt = (bit_cxt as usize) & (N2_BIT - 1);
        let bucket = ((p as usize) >> 11).min(N2_BUCKET - 1);
        let i = (c * N2_BIT + bxt) * N2_BUCKET + bucket;
        self.last_i = i;
        let (n0, n1) = unpack_counts(self.counts[i]);
        let ps = laplace_prob(n0, n1);
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
        learn_packed(&mut self.counts, self.last_i, bit);
    }
}
