//! Hashed multi-order context models (orders 0–6) with integer confidence mixing.
//! No FP, no HashMap iteration, no threads.

use crate::model::Model;

/// Hash-table size masks (power-of-two tables).
const O3_MASK: usize = (1 << 22) - 1; // 4M slots
const O4_MASK: usize = (1 << 22) - 1;
const O5_MASK: usize = (1 << 21) - 1; // 2M slots
const O6_MASK: usize = (1 << 21) - 1;

const O1_STATES: usize = 256 * 256; // byte × bit_cxt
const O2_STATES: usize = 65536 * 256;

/// Fixed per-order bias (higher orders get more weight when equally confident).
const ORDER_BIAS: [u32; 7] = [1, 2, 4, 6, 8, 10, 12];

#[inline]
fn pack_prob(n0: u8, n1: u8) -> u16 {
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
fn update_counts(n0: &mut u8, n1: &mut u8, bit: u8) {
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
fn conf_weight(n0: u8, n1: u8, bias: u32) -> u32 {
    let t = n0 as u32 + n1 as u32;
    // (t+1)^2 * bias — confident contexts dominate.
    (t + 1) * (t + 1) * bias
}

#[inline]
fn mix7(ps: [u16; 7], ws: [u32; 7]) -> u16 {
    let mut num: u64 = 0;
    let mut den: u64 = 0;
    for i in 0..7 {
        let w = ws[i] as u64;
        num += ps[i] as u64 * w;
        den += w;
    }
    let p = (num / den) as u32;
    if p < 1 {
        1
    } else if p > 65534 {
        65534
    } else {
        p as u16
    }
}

/// Composite model: exact order-0/1/2 + hashed order-3..6, confidence-mixed.
pub struct OrdersModel {
    /// hist[0] = most recent completed byte, hist[5] = oldest of the six.
    hist: [u8; 6],
    bit_cxt: u16,
    /// Polynomial hashes of the last o bytes for o = 3..6.
    h3: u32,
    h4: u32,
    h5: u32,
    h6: u32,

    o0_n0: Box<[u8]>,
    o0_n1: Box<[u8]>,
    o1_n0: Box<[u8]>,
    o1_n1: Box<[u8]>,
    o2_n0: Box<[u8]>,
    o2_n1: Box<[u8]>,
    o3_n0: Box<[u8]>,
    o3_n1: Box<[u8]>,
    o4_n0: Box<[u8]>,
    o4_n1: Box<[u8]>,
    o5_n0: Box<[u8]>,
    o5_n1: Box<[u8]>,
    o6_n0: Box<[u8]>,
    o6_n1: Box<[u8]>,
}

impl OrdersModel {
    pub fn new() -> Self {
        Self {
            hist: [0; 6],
            bit_cxt: 1,
            h3: 0,
            h4: 0,
            h5: 0,
            h6: 0,
            o0_n0: vec![0u8; 256].into_boxed_slice(),
            o0_n1: vec![0u8; 256].into_boxed_slice(),
            o1_n0: vec![0u8; O1_STATES].into_boxed_slice(),
            o1_n1: vec![0u8; O1_STATES].into_boxed_slice(),
            o2_n0: vec![0u8; O2_STATES].into_boxed_slice(),
            o2_n1: vec![0u8; O2_STATES].into_boxed_slice(),
            o3_n0: vec![0u8; O3_MASK + 1].into_boxed_slice(),
            o3_n1: vec![0u8; O3_MASK + 1].into_boxed_slice(),
            o4_n0: vec![0u8; O4_MASK + 1].into_boxed_slice(),
            o4_n1: vec![0u8; O4_MASK + 1].into_boxed_slice(),
            o5_n0: vec![0u8; O5_MASK + 1].into_boxed_slice(),
            o5_n1: vec![0u8; O5_MASK + 1].into_boxed_slice(),
            o6_n0: vec![0u8; O6_MASK + 1].into_boxed_slice(),
            o6_n1: vec![0u8; O6_MASK + 1].into_boxed_slice(),
        }
    }

    #[inline]
    fn o0_idx(&self) -> usize {
        self.bit_cxt as usize
    }

    #[inline]
    fn o1_idx(&self) -> usize {
        (self.hist[0] as usize) * 256 + (self.bit_cxt as usize)
    }

    #[inline]
    fn o2_idx(&self) -> usize {
        let o2 = ((self.hist[1] as usize) << 8) | (self.hist[0] as usize);
        o2 * 256 + (self.bit_cxt as usize)
    }

    #[inline]
    fn hash_idx(h: u32, bit_cxt: u16, mask: usize, salt: u32) -> usize {
        let x = h
            .wrapping_mul(0x9E37_79B1)
            .wrapping_add((bit_cxt as u32).wrapping_mul(0x85EB_CA6B))
            .wrapping_add(salt);
        (x as usize) & mask
    }

    #[inline]
    fn indices(&self) -> [usize; 7] {
        let bc = self.bit_cxt;
        [
            self.o0_idx(),
            self.o1_idx(),
            self.o2_idx(),
            Self::hash_idx(self.h3, bc, O3_MASK, 0x1111_1111),
            Self::hash_idx(self.h4, bc, O4_MASK, 0x2222_2222),
            Self::hash_idx(self.h5, bc, O5_MASK, 0x3333_3333),
            Self::hash_idx(self.h6, bc, O6_MASK, 0x4444_4444),
        ]
    }

    #[inline]
    fn shift_hist(&mut self, byte: u8) {
        self.hist[5] = self.hist[4];
        self.hist[4] = self.hist[3];
        self.hist[3] = self.hist[2];
        self.hist[2] = self.hist[1];
        self.hist[1] = self.hist[0];
        self.hist[0] = byte;

        let b0 = self.hist[0] as u32;
        let b1 = self.hist[1] as u32;
        let b2 = self.hist[2] as u32;
        let b3 = self.hist[3] as u32;
        let b4 = self.hist[4] as u32;
        let b5 = self.hist[5] as u32;
        self.h3 =
            b0.wrapping_add(257u32.wrapping_mul(b1.wrapping_add(257u32.wrapping_mul(b2))));
        self.h4 = b0.wrapping_add(257u32.wrapping_mul(
            b1.wrapping_add(257u32.wrapping_mul(b2.wrapping_add(257u32.wrapping_mul(b3)))),
        ));
        self.h5 = b0.wrapping_add(257u32.wrapping_mul(b1.wrapping_add(
            257u32.wrapping_mul(
                b2.wrapping_add(257u32.wrapping_mul(b3.wrapping_add(257u32.wrapping_mul(b4)))),
            ),
        )));
        self.h6 = b0.wrapping_add(257u32.wrapping_mul(b1.wrapping_add(257u32.wrapping_mul(
            b2.wrapping_add(257u32.wrapping_mul(
                b3.wrapping_add(257u32.wrapping_mul(b4.wrapping_add(257u32.wrapping_mul(b5)))),
            )),
        ))));
    }
}

impl Default for OrdersModel {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for OrdersModel {
    #[inline]
    fn predict(&mut self) -> u16 {
        let ix = self.indices();
        let n0s = [
            self.o0_n0[ix[0]],
            self.o1_n0[ix[1]],
            self.o2_n0[ix[2]],
            self.o3_n0[ix[3]],
            self.o4_n0[ix[4]],
            self.o5_n0[ix[5]],
            self.o6_n0[ix[6]],
        ];
        let n1s = [
            self.o0_n1[ix[0]],
            self.o1_n1[ix[1]],
            self.o2_n1[ix[2]],
            self.o3_n1[ix[3]],
            self.o4_n1[ix[4]],
            self.o5_n1[ix[5]],
            self.o6_n1[ix[6]],
        ];
        let mut ps = [0u16; 7];
        let mut ws = [0u32; 7];
        for i in 0..7 {
            ps[i] = pack_prob(n0s[i], n1s[i]);
            ws[i] = conf_weight(n0s[i], n1s[i], ORDER_BIAS[i]);
        }
        mix7(ps, ws)
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        let ix = self.indices();

        update_counts(&mut self.o0_n0[ix[0]], &mut self.o0_n1[ix[0]], bit);
        update_counts(&mut self.o1_n0[ix[1]], &mut self.o1_n1[ix[1]], bit);
        update_counts(&mut self.o2_n0[ix[2]], &mut self.o2_n1[ix[2]], bit);
        update_counts(&mut self.o3_n0[ix[3]], &mut self.o3_n1[ix[3]], bit);
        update_counts(&mut self.o4_n0[ix[4]], &mut self.o4_n1[ix[4]], bit);
        update_counts(&mut self.o5_n0[ix[5]], &mut self.o5_n1[ix[5]], bit);
        update_counts(&mut self.o6_n0[ix[6]], &mut self.o6_n1[ix[6]], bit);

        self.bit_cxt = (self.bit_cxt << 1) | (bit as u16);
        if self.bit_cxt >= 256 {
            let byte = (self.bit_cxt & 0xff) as u8;
            self.shift_hist(byte);
            self.bit_cxt = 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_predict_update_lockstep() {
        let bits: Vec<u8> = (0u8..128)
            .flat_map(|b| (0..8).rev().map(move |i| (b >> i) & 1))
            .collect();

        let mut a = OrdersModel::new();
        let mut b = OrdersModel::new();
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
    fn orders_learns_bias() {
        let mut m = OrdersModel::new();
        let first = m.predict();
        for _ in 0..256 {
            let _ = m.predict();
            m.update(0);
        }
        let later = m.predict();
        assert!(
            later < first,
            "expected p1 to drop after many zeros: {later} vs {first}"
        );
    }

    #[test]
    fn orders_prob_clamped() {
        assert!(pack_prob(0, 0) >= 1 && pack_prob(0, 0) <= 65534);
        assert!(pack_prob(255, 0) >= 1);
        assert!(pack_prob(0, 255) <= 65534);
    }
}
