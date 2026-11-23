//! Context models. Integer / fixed-point only; no FP, no HashMap.

/// Bit predictor: returns p(bit=1) on a 0..=65535 scale, then learns `bit`.
pub trait Model {
    fn predict(&mut self) -> u16;
    fn update(&mut self, bit: u8);
}

const O2: usize = 65536;
const BIT_STATES: usize = 256;

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
    fn prob(n0: u8, n1: u8) -> u16 {
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
        Self::prob(self.n0[i], self.n1[i])
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        let i = self.index();
        if bit == 0 {
            let v = self.n0[i].saturating_add(1);
            self.n0[i] = v;
        } else {
            let v = self.n1[i].saturating_add(1);
            self.n1[i] = v;
        }
        // Rescale for adaptation when counts get large.
        if (self.n0[i] as u16) + (self.n1[i] as u16) >= 250 {
            self.n0[i] >>= 1;
            self.n1[i] >>= 1;
        }

        self.bit_cxt = (self.bit_cxt << 1) | (bit as u16);
        if self.bit_cxt >= 256 {
            let byte = (self.bit_cxt & 0xff) as u8;
            self.prev1 = self.prev0;
            self.prev0 = byte;
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
