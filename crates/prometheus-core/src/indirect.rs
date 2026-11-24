//! Indirect / record-style context model (cmix-class).
//!
//! Stores the last byte seen after order-1 and order-2 contexts, then
//! predicts from hashed (context × last-byte × bit path). Integer only.

use crate::model::Model;
use crate::params::{INDIRECT_BITS, ORDER2_RESCALE};

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
    n0: Box<[u8]>,
    n1: Box<[u8]>,
    mask: usize,
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
            n0: vec![0u8; size].into_boxed_slice(),
            n1: vec![0u8; size].into_boxed_slice(),
            mask: size - 1,
        }
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
    fn hash(&self) -> usize {
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
        (h as usize) & self.mask
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
        let i = self.hash();
        Self::prob(self.n0[i], self.n1[i])
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        let i = self.hash();
        if bit == 0 {
            self.n0[i] = self.n0[i].saturating_add(1);
        } else {
            self.n1[i] = self.n1[i].saturating_add(1);
        }
        if (self.n0[i] as u16) + (self.n1[i] as u16) >= ORDER2_RESCALE {
            self.n0[i] >>= 1;
            self.n1[i] >>= 1;
        }

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
