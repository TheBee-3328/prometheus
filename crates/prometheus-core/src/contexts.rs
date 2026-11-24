//! Hashed higher-order contexts (o3–o6) with checksum collision detection.

use crate::model::Model;
use crate::params::{
    HI_BIAS, HI_MIX_MAX, ORDERS_RESCALE, O3_BITS, O4_BITS, O5_BITS, O6_BITS,
};

const O3_MASK: usize = (1 << O3_BITS) - 1;
const O4_MASK: usize = (1 << O4_BITS) - 1;
const O5_MASK: usize = (1 << O5_BITS) - 1;
const O6_MASK: usize = (1 << O6_BITS) - 1;

#[inline]
fn pack_prob(n0: u8, n1: u8) -> u16 {
    let a = n1 as u32 + 1;
    let b = n0 as u32 + n1 as u32 + 2;
    let p = (a * 65535) / b;
    if p < 1 { 1 } else if p > 65534 { 65534 } else { p as u16 }
}

#[inline]
fn update_counts(n0: &mut u8, n1: &mut u8, bit: u8) {
    if bit == 0 { *n0 = n0.saturating_add(1); } else { *n1 = n1.saturating_add(1); }
    if (*n0 as u16) + (*n1 as u16) >= ORDERS_RESCALE { *n0 >>= 1; *n1 >>= 1; }
}

#[inline]
fn conf_weight(n0: u8, n1: u8, bias: u32) -> u32 {
    let t = n0 as u32 + n1 as u32;
    if t == 0 { 0 } else { t * t * bias }
}

#[inline]
fn mix4(ps: [u16; 4], ws: [u32; 4]) -> (u16, u32) {
    let mut num: u64 = 0;
    let mut den: u64 = 0;
    for i in 0..4 {
        let w = ws[i] as u64;
        num += ps[i] as u64 * w;
        den += w;
    }
    if den == 0 { return (32768, 0); }
    let p = (num / den) as u32;
    let p = if p < 1 { 1 } else if p > 65534 { 65534 } else { p as u16 };
    (p, den.min(u32::MAX as u64) as u32)
}

#[inline]
fn resolve(n0: &mut u8, n1: &mut u8, chk: &mut u8, want: u8) {
    if *chk != want { *n0 = 0; *n1 = 0; *chk = want; }
}

pub struct HashedOrders {
    hist: [u8; 6],
    bit_cxt: u16,
    h3: u32, h4: u32, h5: u32, h6: u32,
    o3_n0: Box<[u8]>, o3_n1: Box<[u8]>, o3_chk: Box<[u8]>,
    o4_n0: Box<[u8]>, o4_n1: Box<[u8]>, o4_chk: Box<[u8]>,
    o5_n0: Box<[u8]>, o5_n1: Box<[u8]>, o5_chk: Box<[u8]>,
    o6_n0: Box<[u8]>, o6_n1: Box<[u8]>, o6_chk: Box<[u8]>,
    last_conf: u32,
}

impl HashedOrders {
    pub fn new() -> Self {
        Self {
            hist: [0; 6], bit_cxt: 1, h3: 0, h4: 0, h5: 0, h6: 0,
            o3_n0: vec![0u8; O3_MASK + 1].into_boxed_slice(),
            o3_n1: vec![0u8; O3_MASK + 1].into_boxed_slice(),
            o3_chk: vec![0u8; O3_MASK + 1].into_boxed_slice(),
            o4_n0: vec![0u8; O4_MASK + 1].into_boxed_slice(),
            o4_n1: vec![0u8; O4_MASK + 1].into_boxed_slice(),
            o4_chk: vec![0u8; O4_MASK + 1].into_boxed_slice(),
            o5_n0: vec![0u8; O5_MASK + 1].into_boxed_slice(),
            o5_n1: vec![0u8; O5_MASK + 1].into_boxed_slice(),
            o5_chk: vec![0u8; O5_MASK + 1].into_boxed_slice(),
            o6_n0: vec![0u8; O6_MASK + 1].into_boxed_slice(),
            o6_n1: vec![0u8; O6_MASK + 1].into_boxed_slice(),
            o6_chk: vec![0u8; O6_MASK + 1].into_boxed_slice(),
            last_conf: 0,
        }
    }
    #[inline]
    pub fn mix_weight_vs_o2(&self) -> u32 {
        let c = self.last_conf;
        let w = (c * HI_MIX_MAX) / (c + 64);
        w.min(HI_MIX_MAX)
    }
    #[inline]
    fn hash_slot(h: u32, bit_cxt: u16, mask: usize, bits: usize, salt: u32) -> (usize, u8) {
        let x = h.wrapping_mul(0x9E37_79B1)
            .wrapping_add((bit_cxt as u32).wrapping_mul(0x85EB_CA6B))
            .wrapping_add(salt);
        ((x as usize) & mask, ((x >> bits) as u8).wrapping_add(1))
    }
    #[inline]
    fn slots(&self) -> [(usize, u8); 4] {
        let bc = self.bit_cxt;
        [
            Self::hash_slot(self.h3, bc, O3_MASK, O3_BITS, 0x1111_1111),
            Self::hash_slot(self.h4, bc, O4_MASK, O4_BITS, 0x2222_2222),
            Self::hash_slot(self.h5, bc, O5_MASK, O5_BITS, 0x3333_3333),
            Self::hash_slot(self.h6, bc, O6_MASK, O6_BITS, 0x4444_4444),
        ]
    }
    #[inline]
    fn shift_hist(&mut self, byte: u8) {
        self.hist = [byte, self.hist[0], self.hist[1], self.hist[2], self.hist[3], self.hist[4]];
        let b = [self.hist[0] as u32, self.hist[1] as u32, self.hist[2] as u32,
                 self.hist[3] as u32, self.hist[4] as u32, self.hist[5] as u32];
        self.h3 = b[0].wrapping_add(257u32.wrapping_mul(b[1].wrapping_add(257u32.wrapping_mul(b[2]))));
        self.h4 = b[0].wrapping_add(257u32.wrapping_mul(b[1].wrapping_add(257u32.wrapping_mul(b[2].wrapping_add(257u32.wrapping_mul(b[3]))))));
        self.h5 = b[0].wrapping_add(257u32.wrapping_mul(b[1].wrapping_add(257u32.wrapping_mul(b[2].wrapping_add(257u32.wrapping_mul(b[3].wrapping_add(257u32.wrapping_mul(b[4]))))))));
        self.h6 = b[0].wrapping_add(257u32.wrapping_mul(b[1].wrapping_add(257u32.wrapping_mul(b[2].wrapping_add(257u32.wrapping_mul(b[3].wrapping_add(257u32.wrapping_mul(b[4].wrapping_add(257u32.wrapping_mul(b[5]))))))))));
    }
}

impl Default for HashedOrders { fn default() -> Self { Self::new() } }

impl Model for HashedOrders {
    #[inline]
    fn predict(&mut self) -> u16 {
        let s = self.slots();
        resolve(&mut self.o3_n0[s[0].0], &mut self.o3_n1[s[0].0], &mut self.o3_chk[s[0].0], s[0].1);
        resolve(&mut self.o4_n0[s[1].0], &mut self.o4_n1[s[1].0], &mut self.o4_chk[s[1].0], s[1].1);
        resolve(&mut self.o5_n0[s[2].0], &mut self.o5_n1[s[2].0], &mut self.o5_chk[s[2].0], s[2].1);
        resolve(&mut self.o6_n0[s[3].0], &mut self.o6_n1[s[3].0], &mut self.o6_chk[s[3].0], s[3].1);
        let n0s = [self.o3_n0[s[0].0], self.o4_n0[s[1].0], self.o5_n0[s[2].0], self.o6_n0[s[3].0]];
        let n1s = [self.o3_n1[s[0].0], self.o4_n1[s[1].0], self.o5_n1[s[2].0], self.o6_n1[s[3].0]];
        let mut ps = [0u16; 4];
        let mut ws = [0u32; 4];
        for i in 0..4 {
            ps[i] = pack_prob(n0s[i], n1s[i]);
            ws[i] = conf_weight(n0s[i], n1s[i], HI_BIAS[i]);
        }
        let (p, conf) = mix4(ps, ws);
        self.last_conf = conf;
        p
    }
    #[inline]
    fn update(&mut self, bit: u8) {
        let s = self.slots();
        resolve(&mut self.o3_n0[s[0].0], &mut self.o3_n1[s[0].0], &mut self.o3_chk[s[0].0], s[0].1);
        resolve(&mut self.o4_n0[s[1].0], &mut self.o4_n1[s[1].0], &mut self.o4_chk[s[1].0], s[1].1);
        resolve(&mut self.o5_n0[s[2].0], &mut self.o5_n1[s[2].0], &mut self.o5_chk[s[2].0], s[2].1);
        resolve(&mut self.o6_n0[s[3].0], &mut self.o6_n1[s[3].0], &mut self.o6_chk[s[3].0], s[3].1);
        update_counts(&mut self.o3_n0[s[0].0], &mut self.o3_n1[s[0].0], bit);
        update_counts(&mut self.o4_n0[s[1].0], &mut self.o4_n1[s[1].0], bit);
        update_counts(&mut self.o5_n0[s[2].0], &mut self.o5_n1[s[2].0], bit);
        update_counts(&mut self.o6_n0[s[3].0], &mut self.o6_n1[s[3].0], bit);
        self.bit_cxt = (self.bit_cxt << 1) | (bit as u16);
        if self.bit_cxt >= 256 {
            let byte = (self.bit_cxt & 0xff) as u8;
            self.shift_hist(byte);
            self.bit_cxt = 1;
        }
    }
}
