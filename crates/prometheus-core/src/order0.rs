//! Order-0 (unigram) bit model with bit path. Integer only.

use crate::model::Model;
use crate::params::ORDER2_RESCALE;

const BIT_STATES: usize = 256;

/// Stationary order-0: index = bit_cxt only.
pub struct Order0Model {
    bit_cxt: u16,
    n0: Box<[u8]>,
    n1: Box<[u8]>,
}

impl Order0Model {
    pub fn new() -> Self {
        Self {
            bit_cxt: 1,
            n0: vec![0u8; BIT_STATES].into_boxed_slice(),
            n1: vec![0u8; BIT_STATES].into_boxed_slice(),
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
}

impl Default for Order0Model {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for Order0Model {
    #[inline]
    fn predict(&mut self) -> u16 {
        let i = self.bit_cxt as usize;
        Self::prob(self.n0[i], self.n1[i])
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        let i = self.bit_cxt as usize;
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
            self.bit_cxt = 1;
        }
    }
}
