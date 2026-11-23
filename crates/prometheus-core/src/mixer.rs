//! Logistic mixer with selectable weight sets and optional two-layer path.
//! Integer / fixed-point only. Weight scale: init ~65536 total, dot >> 16.

use crate::apm::{squash, stretch};

/// One logistic mixer layer: `n_inputs` stretched probs → squash(dot(w, x) >> 16).
pub struct LogisticMixer {
    n_inputs: usize,
    n_sets: usize,
    weights: Box<[i32]>,
    tx: Box<[i32]>,
    set: usize,
    pr: u16,
    rate: i32,
}

impl LogisticMixer {
    pub fn new(n_inputs: usize, n_sets: usize) -> Self {
        assert!(n_inputs >= 1 && n_sets >= 1);
        let init = 65536 / n_inputs as i32;
        let init_w: Vec<i32> = (0..n_inputs).map(|_| init).collect();
        Self::with_init_weights(n_sets, &init_w)
    }

    /// Create with per-input initial weights (cloned into every set).
    pub fn with_init_weights(n_sets: usize, init_w: &[i32]) -> Self {
        let n_inputs = init_w.len();
        assert!(n_inputs >= 1 && n_sets >= 1);
        let mut weights = vec![0i32; n_sets * n_inputs].into_boxed_slice();
        for s in 0..n_sets {
            for i in 0..n_inputs {
                weights[s * n_inputs + i] = init_w[i];
            }
        }
        Self {
            n_inputs,
            n_sets,
            weights,
            tx: vec![0i32; n_inputs].into_boxed_slice(),
            set: 0,
            pr: 2048,
            rate: 5,
        }
    }

    pub fn with_rate(mut self, rate: i32) -> Self {
        self.rate = rate.clamp(1, 16);
        self
    }

    #[inline]
    pub fn select(&mut self, set_cxt: usize) {
        self.set = set_cxt % self.n_sets;
    }

    pub fn predict(&mut self, inputs: &[u16]) -> u16 {
        debug_assert_eq!(inputs.len(), self.n_inputs);
        let base = self.set * self.n_inputs;
        let mut sum: i64 = 0;
        for i in 0..self.n_inputs {
            let t = stretch(inputs[i]);
            self.tx[i] = t;
            sum += self.weights[base + i] as i64 * t as i64;
        }
        let d = (sum >> 16).clamp(-2047, 2047) as i32;
        self.pr = squash(d);
        self.pr
    }

    pub fn update(&mut self, bit: u8) {
        let target = (bit as i32) << 12;
        let err = (target - self.pr as i32) * self.rate;
        let base = self.set * self.n_inputs;
        for i in 0..self.n_inputs {
            let delta = ((err as i64 * self.tx[i] as i64) >> 16) as i32;
            let w = &mut self.weights[base + i];
            *w = (*w + delta).clamp(-16384, 16384);
        }
    }

    pub fn last_p(&self) -> u16 {
        self.pr
    }
}

/// Two-layer logistic mixer for 3 inputs (o0, o1, o2):
/// L1a(o0,o1), L1b(o1,o2 biased), L2(L1a, L1b, o2) with o2 bias.
pub struct TwoLayerMixer {
    pub l1a: LogisticMixer,
    pub l1b: LogisticMixer,
    pub l2: LogisticMixer,
    y1a: u16,
    y1b: u16,
    x2: u16,
}

impl TwoLayerMixer {
    pub fn new(n_sets: usize) -> Self {
        Self {
            // Mild o1 bias over o0.
            l1a: LogisticMixer::with_init_weights(n_sets, &[21845, 43691]).with_rate(5),
            // Strong o2 bias over o1.
            l1b: LogisticMixer::with_init_weights(n_sets, &[16384, 49152]).with_rate(5),
            // Prefer raw o2, then L1b, then L1a.
            l2: LogisticMixer::with_init_weights(n_sets, &[8192, 16384, 40960]).with_rate(4),
            y1a: 2048,
            y1b: 2048,
            x2: 2048,
        }
    }

    pub fn predict(&mut self, p: [u16; 3], set_cxt: usize) -> u16 {
        self.l1a.select(set_cxt);
        self.l1b.select(set_cxt);
        self.l2.select(set_cxt);
        self.y1a = self.l1a.predict(&[p[0], p[1]]);
        self.y1b = self.l1b.predict(&[p[1], p[2]]);
        self.x2 = p[2];
        self.l2.predict(&[self.y1a, self.y1b, self.x2])
    }

    pub fn update(&mut self, bit: u8) {
        self.l2.update(bit);
        self.l1a.update(bit);
        self.l1b.update(bit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_inputs_pass_through() {
        let mut m = LogisticMixer::new(3, 1);
        for &p in &[512u16, 1024, 2048, 3000, 3800] {
            let out = m.predict(&[p, p, p]);
            let diff = (out as i32 - p as i32).abs();
            assert!(diff <= 8, "passthrough p={p} out={out} diff={diff}");
        }
    }

    #[test]
    fn biased_init_favors_last() {
        let mut m = LogisticMixer::with_init_weights(1, &[4096, 12288, 49152]);
        // Disagree: last input sharp high, others low.
        let out = m.predict(&[800, 800, 3800]);
        assert!(out > 2500, "expected bias toward high input, got {out}");
    }

    #[test]
    fn mixer_learns_better_input() {
        let mut m = LogisticMixer::new(2, 1);
        for _ in 0..400 {
            let _ = m.predict(&[1024, 3686]);
            m.update(1);
        }
        assert!(
            m.weights[1] > m.weights[0],
            "w0={} w1={}",
            m.weights[0],
            m.weights[1]
        );
    }

    #[test]
    fn mixer_weight_sets_independent() {
        let mut m = LogisticMixer::new(2, 4);
        m.select(0);
        for _ in 0..50 {
            let _ = m.predict(&[1000, 3000]);
            m.update(1);
        }
        let w0 = m.weights[0];
        m.select(1);
        let init = 65536 / 2;
        assert_eq!(m.weights[2], init);
        assert_ne!(w0, init);
    }

    #[test]
    fn two_layer_lockstep() {
        let mut a = TwoLayerMixer::new(8);
        let mut b = TwoLayerMixer::new(8);
        let seq = [
            ([1000u16, 2000, 3000], 0u8),
            ([1500, 2500, 2000], 1),
            ([800, 900, 3500], 1),
            ([2048, 2048, 2048], 0),
        ];
        for &(p, bit) in &seq {
            assert_eq!(a.predict(p, 3), b.predict(p, 3));
            a.update(bit);
            b.update(bit);
        }
    }

    #[test]
    fn mixer_predict_in_range() {
        let mut m = LogisticMixer::new(3, 2);
        m.select(1);
        let p = m.predict(&[1, 2048, 4094]);
        assert!(p <= 4095);
    }
}
