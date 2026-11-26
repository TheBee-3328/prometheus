//! Indirect / record-style context model (cmix-class).
//!
//! Stores the last byte seen after order-1 and order-2 contexts, then
//! predicts from hashed (context × last-byte × bit path). Integer only.

use crate::model::Model;
use crate::params::{INDIRECT_BITS, ORDER2_RESCALE};

#[inline]
fn pack_counts(n0: u8, n1: u8) -> u16 {
    (n0 as u16) | ((n1 as u16) << 8)
}

#[inline]
fn unpack_counts(c: u16) -> (u8, u8) {
    (c as u8, (c >> 8) as u8)
}

/// Last-byte-after-context predictor.
pub struct IndirectModel {
    prev0: u8,
    prev1: u8,
    prev2: u8,
    bit_cxt: u16,
    /// Last byte after each order-1 context.
    t1: [u8; 256],
    /// Last byte after each order-2 context.
    t2: Box<[u8]>,
    /// Last byte after hashed order-3 context.
    t3: Box<[u8]>,
    /// Packed n0/n1 (lo8=n0, hi8=n1).
    counts: Box<[u16]>,
    /// 8-bit context fingerprint; 0 = empty.
    chk: Box<[u8]>,
    mask: usize,
    last_i: usize,
    last_cs: u8,
}

impl IndirectModel {
    pub fn new() -> Self {
        let size = 1usize << INDIRECT_BITS;
        Self {
            prev0: 0,
            prev1: 0,
            prev2: 0,
            bit_cxt: 1,
            t1: [0u8; 256],
            t2: vec![0u8; 65536].into_boxed_slice(),
            t3: vec![0u8; 65536].into_boxed_slice(),
            counts: vec![0u16; size].into_boxed_slice(),
            chk: vec![0u8; size].into_boxed_slice(),
            mask: size - 1,
            last_i: 0,
            last_cs: 0,
        }
    }

    /// Last byte seen after the current order-1 context (for cross models).
    #[inline]
    pub fn ind1(&self) -> u8 {
        self.t1[self.prev0 as usize]
    }

    /// Last byte after current order-2 context (for richer cross).
    #[inline]
    pub fn ind2(&self) -> u8 {
        let o2 = ((self.prev1 as usize) << 8) | (self.prev0 as usize);
        self.t2[o2]
    }

    #[inline]
    fn o3_idx(&self) -> usize {
        let mut h: u32 = 2166136261;
        h ^= self.prev0 as u32;
        h = h.wrapping_mul(16777619);
        h ^= self.prev1 as u32;
        h = h.wrapping_mul(16777619);
        h ^= self.prev2 as u32;
        h = h.wrapping_mul(16777619);
        (h as usize) & 65535
    }

    #[inline]
    fn hash_cs(&self) -> (usize, u8) {
        let o2 = ((self.prev1 as usize) << 8) | (self.prev0 as usize);
        let ind1 = self.t1[self.prev0 as usize] as u32;
        let ind2 = self.t2[o2] as u32;
        let ind3 = self.t3[self.o3_idx()] as u32;
        let mut h: u32 = 2166136261;
        h ^= self.bit_cxt as u32;
        h = h.wrapping_mul(16777619);
        h ^= self.prev0 as u32;
        h = h.wrapping_mul(16777619);
        h ^= ind1;
        h = h.wrapping_mul(16777619);
        h ^= ind2.wrapping_mul(0x9e3779b9);
        h = h.wrapping_mul(16777619);
        h ^= ind3.wrapping_mul(0x85ebca6b);
        h = h.wrapping_mul(16777619);
        let idx = (h as usize) & self.mask;
        let mut cs = (h.wrapping_mul(0xc2b2ae35) >> 24) as u8;
        if cs == 0 {
            cs = 1;
        }
        (idx, cs)
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

impl Default for IndirectModel {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for IndirectModel {
    #[inline]
    fn predict(&mut self) -> u16 {
        let (i, cs) = self.hash_cs();
        self.last_i = i;
        self.last_cs = cs;
        if self.chk[i] != cs {
            return 32768;
        }
        let (n0, n1) = unpack_counts(self.counts[i]);
        Self::prob(n0, n1)
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        let i = self.last_i;
        let cs = self.last_cs;
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

        self.bit_cxt = (self.bit_cxt << 1) | (bit as u16);
        if self.bit_cxt >= 256 {
            let byte = (self.bit_cxt & 0xff) as u8;
            let o2 = ((self.prev1 as usize) << 8) | (self.prev0 as usize);
            let o3 = self.o3_idx();
            self.t1[self.prev0 as usize] = byte;
            self.t2[o2] = byte;
            self.t3[o3] = byte;
            self.prev2 = self.prev1;
            self.prev1 = self.prev0;
            self.prev0 = byte;
            self.bit_cxt = 1;
        }
    }
}
