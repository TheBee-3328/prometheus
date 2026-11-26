//! Secondary symbol estimation (SSE / light APM). Integer only.
//!
//! Refines a primary probability using (bit_cxt, quantized p) counts.
//! Counts are packed as `u16 = n0 | (n1 << 8)` so each slot is one cache line touch.
//! SSE_MODE selects Laplace / StateMap / richer cxt / warm-gate / fine buckets / interp.

use crate::apm::{p12_to_p16, p16_to_p12, Apm};
use crate::params::{ORDER2_RESCALE, SSE_MODE};
use crate::statemap::{sm_init, sm_predict, sm_update};

/// bit_cxt (8) × match_bucket (8) = 2048 contexts
const N_CXT: usize = 2048;
/// Rich mode: bit_cxt × mb × last_nibble (16) = 32768 contexts
const N_CXT_RICH: usize = 32768;
/// Full last_byte × bit_cxt × mb = 256*256*8 = 524288 contexts
const N_CXT_BYTE: usize = 524288;
const N_BUCKET: usize = 64; // p >> 10
const N_BUCKET_FINE: usize = 128; // p >> 9
const SIZE: usize = N_CXT * N_BUCKET;
const SIZE_FINE: usize = N_CXT * N_BUCKET_FINE;
const SIZE_RICH: usize = N_CXT_RICH * N_BUCKET;
const SIZE_NIB_LAP: usize = N_CXT_RICH * N_BUCKET;
const SIZE_BYTE: usize = N_CXT_BYTE * N_BUCKET; // ~64 MB

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

#[inline]
fn mix_out(p: u16, ps: u16, w: u32) -> u16 {
    let out = (p as u32 * (256 - w) + ps as u32 * w) / 256;
    if out < 1 {
        1
    } else if out > 65534 {
        65534
    } else {
        out as u16
    }
}

/// Adaptive SSE table: mixes primary p with a learned conditional.
pub struct Sse {
    /// Packed n0/n1 per slot (Laplace modes).
    counts: Box<[u16]>,
    /// StateMap cells (SM modes).
    cells: Box<[u32]>,
    /// Interpolated Apm-style SSE (mode 5).
    apm: Option<Apm>,
    /// last index for update
    last_i: usize,
    /// blend of SSE vs primary (0..=255)
    mix_w: u32,
    mode: u32,
}

impl Sse {
    pub fn new(mix_w: u32) -> Self {
        let mode = SSE_MODE;
        let (counts, cells, apm) = match mode {
            1 => (
                Vec::new().into_boxed_slice(),
                vec![sm_init(); SIZE].into_boxed_slice(),
                None,
            ),
            2 => (
                Vec::new().into_boxed_slice(),
                vec![sm_init(); SIZE_RICH].into_boxed_slice(),
                None,
            ),
            4 => (
                vec![0u16; SIZE_FINE].into_boxed_slice(),
                Vec::new().into_boxed_slice(),
                None,
            ),
            5 => (
                Vec::new().into_boxed_slice(),
                Vec::new().into_boxed_slice(),
                Some(Apm::new(N_CXT).with_rate_shift(7)),
            ),
            6 => (
                vec![0u16; SIZE_NIB_LAP].into_boxed_slice(),
                Vec::new().into_boxed_slice(),
                None,
            ),
            7 => (
                vec![0u16; SIZE_BYTE].into_boxed_slice(),
                Vec::new().into_boxed_slice(),
                None,
            ),
            _ => (
                // modes 0, 3
                vec![0u16; SIZE].into_boxed_slice(),
                Vec::new().into_boxed_slice(),
                None,
            ),
        };
        Self {
            counts,
            cells,
            apm,
            last_i: 0,
            mix_w,
            mode,
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

    /// Refine `p` using `bit_cxt`, match-length bucket, and optional last_byte.
    #[inline]
    pub fn refine(&mut self, p: u16, bit_cxt: u16, match_len: u32, last_byte: u8) -> u16 {
        let mb = Self::match_bucket(match_len) as usize;
        match self.mode {
            1 => {
                let cxt = ((bit_cxt as usize) & 255) | (mb << 8);
                let bucket = ((p as usize) >> 10).min(N_BUCKET - 1);
                let i = cxt * N_BUCKET + bucket;
                self.last_i = i;
                mix_out(p, sm_predict(self.cells[i]), self.mix_w)
            }
            2 => {
                let nib = (last_byte as usize) >> 4;
                let cxt = (((bit_cxt as usize) & 255) | (mb << 8)) | (nib << 11);
                let bucket = ((p as usize) >> 10).min(N_BUCKET - 1);
                let i = cxt * N_BUCKET + bucket;
                self.last_i = i;
                mix_out(p, sm_predict(self.cells[i]), self.mix_w)
            }
            3 => {
                let cxt = ((bit_cxt as usize) & 255) | (mb << 8);
                let bucket = ((p as usize) >> 10).min(N_BUCKET - 1);
                let i = cxt * N_BUCKET + bucket;
                self.last_i = i;
                let (n0, n1) = unpack_counts(self.counts[i]);
                let ps = laplace_prob(n0, n1);
                let mass = (n0 as u32) + (n1 as u32);
                let w = if mass < 8 {
                    self.mix_w * mass / 8
                } else {
                    self.mix_w
                };
                mix_out(p, ps, w)
            }
            4 => {
                let cxt = ((bit_cxt as usize) & 255) | (mb << 8);
                let bucket = ((p as usize) >> 9).min(N_BUCKET_FINE - 1);
                let i = cxt * N_BUCKET_FINE + bucket;
                self.last_i = i;
                let (n0, n1) = unpack_counts(self.counts[i]);
                mix_out(p, laplace_prob(n0, n1), self.mix_w)
            }
            5 => {
                let cxt = ((bit_cxt as usize) & 255) | (mb << 8);
                let apm = self.apm.as_mut().unwrap();
                let p12 = apm.predict(p16_to_p12(p), cxt);
                mix_out(p, p12_to_p16(p12), self.mix_w)
            }
            6 => {
                let nib = (last_byte as usize) >> 4;
                let cxt = (((bit_cxt as usize) & 255) | (mb << 8)) | (nib << 11);
                let bucket = ((p as usize) >> 10).min(N_BUCKET - 1);
                let i = cxt * N_BUCKET + bucket;
                self.last_i = i;
                let (n0, n1) = unpack_counts(self.counts[i]);
                mix_out(p, laplace_prob(n0, n1), self.mix_w)
            }
            7 => {
                // last_byte × bit_cxt × mb
                let cxt = ((last_byte as usize) << 11)
                    | (((bit_cxt as usize) & 255) << 3)
                    | mb;
                let bucket = ((p as usize) >> 10).min(N_BUCKET - 1);
                let i = cxt * N_BUCKET + bucket;
                self.last_i = i;
                let (n0, n1) = unpack_counts(self.counts[i]);
                mix_out(p, laplace_prob(n0, n1), self.mix_w)
            }
            _ => {
                let cxt = ((bit_cxt as usize) & 255) | (mb << 8);
                let bucket = ((p as usize) >> 10).min(N_BUCKET - 1);
                let i = cxt * N_BUCKET + bucket;
                self.last_i = i;
                let (n0, n1) = unpack_counts(self.counts[i]);
                mix_out(p, laplace_prob(n0, n1), self.mix_w)
            }
        }
    }

    #[inline]
    pub fn update(&mut self, bit: u8) {
        match self.mode {
            1 | 2 => {
                self.cells[self.last_i] = sm_update(self.cells[self.last_i], bit);
            }
            5 => {
                if let Some(apm) = self.apm.as_mut() {
                    apm.update(bit);
                }
            }
            _ => {
                learn_packed(&mut self.counts, self.last_i, bit);
            }
        }
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
        mix_out(p, ps, self.mix_w)
    }

    #[inline]
    pub fn update(&mut self, bit: u8) {
        learn_packed(&mut self.counts, self.last_i, bit);
    }
}
