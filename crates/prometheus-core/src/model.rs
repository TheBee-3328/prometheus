//! Context models. Integer / fixed-point only; no FP, no HashMap.

use crate::apm::{p12_to_p16, p16_to_p12, Apm};
use crate::mixer::TwoLayerMixer;

/// Bit predictor: returns p(bit=1) on a 0..=65535 scale, then learns `bit`.
pub trait Model {
    fn predict(&mut self) -> u16;
    fn update(&mut self, bit: u8);
}

const O2: usize = 65536;
const BIT_STATES: usize = 256;

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
fn bump_count(n0: &mut u8, n1: &mut u8, bit: u8) {
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

/// How many data bits already seen in current byte (0..7).
#[inline]
fn bits_in_cxt(bit_cxt: u16) -> usize {
    match bit_cxt {
        1 => 0,
        2..=3 => 1,
        4..=7 => 2,
        8..=15 => 3,
        16..=31 => 4,
        32..=63 => 5,
        64..=127 => 6,
        _ => 7,
    }
}

/// Order-0: intra-byte bit path only (256 states).
pub struct Order0Model {
    bit_cxt: u16,
    n0: [u8; BIT_STATES],
    n1: [u8; BIT_STATES],
}

impl Order0Model {
    pub fn new() -> Self {
        Self {
            bit_cxt: 1,
            n0: [0; BIT_STATES],
            n1: [0; BIT_STATES],
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
        laplace_prob(self.n0[i], self.n1[i])
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        let i = self.bit_cxt as usize;
        bump_count(&mut self.n0[i], &mut self.n1[i], bit);
        self.bit_cxt = (self.bit_cxt << 1) | (bit as u16);
        if self.bit_cxt >= 256 {
            self.bit_cxt = 1;
        }
    }
}

/// Order-1: previous byte × intra-byte bit path.
pub struct Order1Model {
    prev0: u8,
    bit_cxt: u16,
    n0: Box<[u8]>,
    n1: Box<[u8]>,
}

impl Order1Model {
    pub fn new() -> Self {
        Self {
            prev0: 0,
            bit_cxt: 1,
            n0: vec![0u8; 256 * BIT_STATES].into_boxed_slice(),
            n1: vec![0u8; 256 * BIT_STATES].into_boxed_slice(),
        }
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
        let i = (self.prev0 as usize) * BIT_STATES + (self.bit_cxt as usize);
        laplace_prob(self.n0[i], self.n1[i])
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        let i = (self.prev0 as usize) * BIT_STATES + (self.bit_cxt as usize);
        bump_count(&mut self.n0[i], &mut self.n1[i], bit);
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
pub struct Order2Model {
    prev0: u8,
    prev1: u8,
    bit_cxt: u16,
    n0: Box<[u8]>,
    n1: Box<[u8]>,
}

impl Order2Model {
    pub fn new() -> Self {
        Self {
            prev0: 0,
            prev1: 0,
            bit_cxt: 1,
            n0: vec![0u8; O2 * BIT_STATES].into_boxed_slice(),
            n1: vec![0u8; O2 * BIT_STATES].into_boxed_slice(),
        }
    }

    #[inline]
    fn index(&self) -> usize {
        let o2 = ((self.prev1 as usize) << 8) | (self.prev0 as usize);
        o2 * BIT_STATES + (self.bit_cxt as usize)
    }

    #[inline]
    pub fn bit_cxt(&self) -> u16 {
        self.bit_cxt
    }

    #[inline]
    pub fn prev0(&self) -> u8 {
        self.prev0
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
        laplace_prob(self.n0[i], self.n1[i])
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        let i = self.index();
        bump_count(&mut self.n0[i], &mut self.n1[i], bit);

        self.bit_cxt = (self.bit_cxt << 1) | (bit as u16);
        if self.bit_cxt >= 256 {
            let byte = (self.bit_cxt & 0xff) as u8;
            self.prev1 = self.prev0;
            self.prev0 = byte;
            self.bit_cxt = 1;
        }
    }
}

/// Mixed model: Order0+Order1+Order2 → two-layer logistic mixer → APM chain.
pub struct MixedModel {
    o0: Order0Model,
    o1: Order1Model,
    o2: Order2Model,
    mixer: TwoLayerMixer,
    apm1: Apm,
    apm2: Apm,
}

impl MixedModel {
    pub fn new() -> Self {
        Self {
            o0: Order0Model::new(),
            o1: Order1Model::new(),
            o2: Order2Model::new(),
            // Weight set = bit position 0..7 (selectable sets).
            mixer: TwoLayerMixer::new(8),
            apm1: Apm::new(256 * 8).with_rate_shift(7),
            apm2: Apm::new(256 * 8).with_rate_shift(8),
        }
    }

    #[inline]
    fn set_cxt(&self) -> usize {
        bits_in_cxt(self.o2.bit_cxt())
    }

    #[inline]
    fn apm1_cxt(&self) -> usize {
        let bp = bits_in_cxt(self.o2.bit_cxt());
        (self.o2.bit_cxt() as usize & 255) * 8 + bp
    }

    #[inline]
    fn apm2_cxt(&self) -> usize {
        let bp = bits_in_cxt(self.o2.bit_cxt());
        (self.o2.prev0() as usize) * 8 + bp
    }
}

impl Default for MixedModel {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for MixedModel {
    fn predict(&mut self) -> u16 {
        let p0 = p16_to_p12(self.o0.predict());
        let p1 = p16_to_p12(self.o1.predict());
        let p2 = p16_to_p12(self.o2.predict());
        let set = self.set_cxt();
        let mixed = self.mixer.predict([p0, p1, p2], set);
        let a1 = self.apm1.predict(mixed, self.apm1_cxt());
        let a2 = self.apm2.predict(a1, self.apm2_cxt());
        p12_to_p16(a2)
    }

    fn update(&mut self, bit: u8) {
        self.apm2.update(bit);
        self.apm1.update(bit);
        self.mixer.update(bit);
        self.o0.update(bit);
        self.o1.update(bit);
        self.o2.update(bit);
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
        let first = m.predict();
        for _ in 0..32 {
            let _ = m.predict();
            m.update(0);
        }
        for _ in 0..64 {
            let _ = m.predict();
            m.update(0);
        }
        let later = m.predict();
        assert!(later < first, "expected p1 to drop after many zeros: {later} vs {first}");
    }

    #[test]
    fn mixed_lockstep() {
        let bits: Vec<u8> = (0u8..128)
            .flat_map(|b| (0..8).rev().map(move |i| (b >> i) & 1))
            .collect();
        let mut a = MixedModel::new();
        let mut b = MixedModel::new();
        for &bit in &bits {
            assert_eq!(a.predict(), b.predict());
            a.update(bit);
            b.update(bit);
        }
    }

    #[test]
    fn mixed_beats_flat_on_zeros() {
        let mut m = MixedModel::new();
        let first = m.predict();
        for _ in 0..256 {
            let _ = m.predict();
            m.update(0);
        }
        let later = m.predict();
        assert!(later < first, "mixed should lower p1 on zeros: {later} vs {first}");
    }

    #[test]
    fn bits_in_cxt_values() {
        assert_eq!(bits_in_cxt(1), 0);
        assert_eq!(bits_in_cxt(2), 1);
        assert_eq!(bits_in_cxt(3), 1);
        assert_eq!(bits_in_cxt(128), 7);
        assert_eq!(bits_in_cxt(255), 7);
    }
}
