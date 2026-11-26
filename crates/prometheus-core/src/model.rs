//! Context models. Integer / fixed-point only; no FP, no HashMap.

use crate::params::ORDER2_RESCALE;
use crate::statemap::{sm_init, sm_predict, sm_update};

/// Bit predictor: returns p(bit=1) on a 0..=65535 scale, then learns `bit`.
pub trait Model {
    fn predict(&mut self) -> u16;
    fn update(&mut self, bit: u8);
}

const O1: usize = 256;
const O2: usize = 65536;
const BIT_STATES: usize = 256;

#[inline]
fn laplace_prob(n0: u8, n1: u8) -> u16 {
    // Laplace +1; clamp away from 0/65535 for the arithmetic coder.
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
fn pack_counts(n0: u8, n1: u8) -> u16 {
    (n0 as u16) | ((n1 as u16) << 8)
}

#[inline]
fn unpack_counts(c: u16) -> (u8, u8) {
    (c as u8, (c >> 8) as u8)
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

/// Order-1 byte context with intra-byte bit path (MSB first).
///
/// Context index = `prev0 * 256 + bit_cxt`.
pub struct Order1Model {
    prev0: u8,
    bit_cxt: u16,
    /// Packed n0/n1 (lo8=n0, hi8=n1) — one touch per slot.
    counts: Box<[u16]>,
    /// Cached index from last `predict` for `update`.
    last_i: usize,
}

impl Order1Model {
    pub fn new() -> Self {
        Self {
            prev0: 0,
            bit_cxt: 1,
            counts: vec![0u16; O1 * BIT_STATES].into_boxed_slice(),
            last_i: 0,
        }
    }

    #[inline]
    fn index(&self) -> usize {
        (self.prev0 as usize) * BIT_STATES + (self.bit_cxt as usize)
    }
}

impl Default for Order1Model {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for Order1Model {
    #[inline]
    fn predict(&mut self) -> u16 {
        let i = self.index();
        self.last_i = i;
        let (n0, n1) = unpack_counts(self.counts[i]);
        laplace_prob(n0, n1)
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        learn_packed(&mut self.counts, self.last_i, bit);
        self.bit_cxt = (self.bit_cxt << 1) | (bit as u16);
        if self.bit_cxt >= 256 {
            self.prev0 = (self.bit_cxt & 0xff) as u8;
            self.bit_cxt = 1;
        }
    }
}

/// Order-2 byte context with intra-byte bit path (MSB first).
///
/// Context index = `(prev1 << 8 | prev0) * 256 + bit_cxt` where `bit_cxt`
/// starts at 1 and grows with each bit until a byte completes.
///
/// Paq-style count-dependent StateMap (p + n) — dual-rate via 3/(n+4) step.
pub struct Order2Model {
    prev0: u8,
    prev1: u8,
    bit_cxt: u16,
    /// Packed StateMap cells: hi16=p, lo8=n.
    cells: Box<[u32]>,
    /// Cached index from last `predict` for `update`.
    last_i: usize,
}

impl Order2Model {
    pub fn new() -> Self {
        Self {
            prev0: 0,
            prev1: 0,
            bit_cxt: 1,
            cells: vec![sm_init(); O2 * BIT_STATES].into_boxed_slice(),
            last_i: 0,
        }
    }

    #[inline]
    fn index(&self) -> usize {
        let o2 = ((self.prev1 as usize) << 8) | (self.prev0 as usize);
        o2 * BIT_STATES + (self.bit_cxt as usize)
    }
}

impl Default for Order2Model {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for Order2Model {
    #[inline]
    fn predict(&mut self) -> u16 {
        let i = self.index();
        self.last_i = i;
        sm_predict(self.cells[i])
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        let i = self.last_i;
        self.cells[i] = sm_update(self.cells[i], bit);

        self.bit_cxt = (self.bit_cxt << 1) | (bit as u16);
        if self.bit_cxt >= 256 {
            let byte = (self.bit_cxt & 0xff) as u8;
            self.prev1 = self.prev0;
            self.prev0 = byte;
            self.bit_cxt = 1;
        }
    }
}

/// Hashed order-N byte context (N≥3) with bit path in the hash.
///
/// Table size = 1 << bits. Each slot stores an 8-bit checksum of the full
/// context hash; mismatch → treat as empty (flat p) and replace on update.
/// `use_statemap`: paq-style p+n cells (not u16 n0/n1 — that was rejected).
pub struct HashOrderModel {
    order: usize,
    bits: usize,
    mask: usize,
    use_statemap: bool,
    /// recent bytes, index 0 = most recent
    hist: [u8; 8],
    bit_cxt: u16,
    /// Packed n0/n1 when not using StateMap (lo8=n0, hi8=n1).
    counts: Box<[u16]>,
    cells: Box<[u32]>,
    /// 8-bit context fingerprint; 0 = empty (never written).
    chk: Box<[u8]>,
    /// Cached (index, checksum) from last `predict` for `update`.
    last_i: usize,
    last_cs: u8,
}

impl HashOrderModel {
    pub fn new(order: usize, bits: usize) -> Self {
        Self::new_with(order, bits, false)
    }

    pub fn new_statemap(order: usize, bits: usize) -> Self {
        Self::new_with(order, bits, true)
    }

    fn new_with(order: usize, bits: usize, use_statemap: bool) -> Self {
        assert!((3..=8).contains(&order));
        assert!((12..=30).contains(&bits));
        let size = 1usize << bits;
        Self {
            order,
            bits,
            mask: size - 1,
            use_statemap,
            hist: [0u8; 8],
            bit_cxt: 1,
            counts: if use_statemap {
                Vec::new().into_boxed_slice()
            } else {
                vec![0u16; size].into_boxed_slice()
            },
            cells: if use_statemap {
                vec![sm_init(); size].into_boxed_slice()
            } else {
                Vec::new().into_boxed_slice()
            },
            chk: vec![0u8; size].into_boxed_slice(),
            last_i: 0,
            last_cs: 0,
        }
    }

    /// Index + non-zero 8-bit checksum (independent of index bits).
    #[inline]
    fn hash_cs(&self) -> (usize, u8) {
        // FNV-ish over hist[0..order) and bit_cxt (bit position in context).
        let mut h: u32 = 2166136261;
        h ^= self.bit_cxt as u32;
        h = h.wrapping_mul(16777619);
        // Manual unroll for common orders (3..=7) — same mul chain as the loop.
        match self.order {
            3 => {
                h ^= self.hist[0] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[1] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[2] as u32;
                h = h.wrapping_mul(16777619);
            }
            4 => {
                h ^= self.hist[0] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[1] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[2] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[3] as u32;
                h = h.wrapping_mul(16777619);
            }
            5 => {
                h ^= self.hist[0] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[1] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[2] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[3] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[4] as u32;
                h = h.wrapping_mul(16777619);
            }
            6 => {
                h ^= self.hist[0] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[1] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[2] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[3] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[4] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[5] as u32;
                h = h.wrapping_mul(16777619);
            }
            7 => {
                h ^= self.hist[0] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[1] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[2] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[3] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[4] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[5] as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.hist[6] as u32;
                h = h.wrapping_mul(16777619);
            }
            _ => {
                for i in 0..self.order {
                    h ^= self.hist[i] as u32;
                    h = h.wrapping_mul(16777619);
                }
            }
        }
        // Mix in order id so o3/o4 tables differ even if shared pattern.
        h ^= (self.order as u32).wrapping_mul(0x9e3779b9);
        let idx = (h as usize) & self.mask;
        // Checksum from a second mix so it is not just discarded index bits.
        let mut cs = (h.wrapping_mul(0x85ebca6b) >> 24) as u8;
        if cs == 0 {
            cs = 1;
        }
        (idx, cs)
    }

    pub fn bits(&self) -> usize {
        self.bits
    }
}

impl Model for HashOrderModel {
    #[inline]
    fn predict(&mut self) -> u16 {
        let (i, cs) = self.hash_cs();
        self.last_i = i;
        self.last_cs = cs;
        if self.chk[i] != cs {
            return 32768;
        }
        if self.use_statemap {
            sm_predict(self.cells[i])
        } else {
            let (n0, n1) = unpack_counts(self.counts[i]);
            laplace_prob(n0, n1)
        }
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        let i = self.last_i;
        let cs = self.last_cs;
        if self.chk[i] != cs {
            // Collision / empty: replace slot (lowest priority = overwrite).
            self.chk[i] = cs;
            if self.use_statemap {
                self.cells[i] = sm_init();
            } else {
                self.counts[i] = 0;
            }
        }
        if self.use_statemap {
            self.cells[i] = sm_update(self.cells[i], bit);
        } else {
            learn_packed(&mut self.counts, i, bit);
        }
        self.bit_cxt = (self.bit_cxt << 1) | (bit as u16);
        if self.bit_cxt >= 256 {
            let byte = (self.bit_cxt & 0xff) as u8;
            // shift history (most recent at 0)
            for k in (1..self.order).rev() {
                self.hist[k] = self.hist[k - 1];
            }
            self.hist[0] = byte;
            self.bit_cxt = 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order2_predict_update_lockstep() {
        let bits: Vec<u8> = (0u8..64)
            .flat_map(|b| (0..8).rev().map(move |i| (b >> i) & 1))
            .collect();

        let mut a = Order2Model::new();
        let mut b = Order2Model::new();
        let mut preds = Vec::new();
        for &bit in &bits {
            let p = a.predict();
            preds.push(p);
            a.update(bit);
        }
        for (i, &bit) in bits.iter().enumerate() {
            let p = b.predict();
            assert_eq!(p, preds[i], "predict mismatch at bit {i}");
            b.update(bit);
        }
    }

    #[test]
    fn order2_learns_bias() {
        let mut m = Order2Model::new();
        // All-zero stream under empty context should drive p(1) down.
        let first = m.predict();
        for _ in 0..32 {
            let _ = m.predict();
            m.update(0);
        }
        // Reset bit path by finishing current byte if needed — keep updating zeros.
        for _ in 0..64 {
            let _ = m.predict();
            m.update(0);
        }
        let later = m.predict();
        assert!(later < first, "expected p1 to drop after many zeros: {later} vs {first}");
    }
}
