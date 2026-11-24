//! Adaptive Probability Map (paq-style APM) with stretch/squash.
//! Integer / fixed-point only. Internal probabilities use 12-bit scale 0..=4095.

use std::sync::OnceLock;

/// Stretch lookup heap-built once (keeps ~8 KB out of the decompressor image).
fn stretch_table() -> &'static [i16] {
    static STRETCH: OnceLock<Box<[i16]>> = OnceLock::new();
    STRETCH
        .get_or_init(|| {
            let mut t = vec![0i16; 4096];
            let mut i = 0usize;
            while i < 4096 {
                let p = if i == 0 {
                    1i32
                } else if i == 4095 {
                    4095i32
                } else {
                    i as i32
                };
                let mut lo = -2047i32;
                let mut hi = 2047i32;
                let mut k = 0;
                while k < 16 {
                    let mid = (lo + hi) / 2;
                    let s = squash_const(mid);
                    if s < p {
                        lo = mid + 1;
                    } else {
                        hi = mid;
                    }
                    k += 1;
                }
                let clamped = if lo < -2047 {
                    -2047
                } else if lo > 2047 {
                    2047
                } else {
                    lo
                };
                t[i] = clamped as i16;
                i += 1;
            }
            t.into_boxed_slice()
        })
        .as_ref()
}

/// Const squash used to build STRETCH (paq8-style knots).
const fn squash_const(d: i32) -> i32 {
    const KNOT: [i32; 33] = [
        1, 2, 3, 6, 10, 16, 27, 45, 73, 120, 194, 310, 488, 747, 1101, 1546, 2047, 2549, 2994,
        3348, 3607, 3785, 3901, 3975, 4022, 4050, 4069, 4080, 4085, 4091, 4092, 4093, 4094,
    ];
    let x = if d < -2047 {
        -2047
    } else if d > 2047 {
        2047
    } else {
        d
    };
    let x = x + 2047; // 0..4094
    let i = (x >> 7) as usize; // 0..31
    let w = x & 127;
    let a = KNOT[i];
    let b = KNOT[i + 1];
    (a * (128 - w) + b * w) >> 7
}

/// Map stretched logit (-2047..2047) → probability 0..4095.
#[inline]
pub fn squash(d: i32) -> u16 {
    let v = squash_const(d);
    if v < 0 {
        0
    } else if v > 4095 {
        4095
    } else {
        v as u16
    }
}

/// Map probability 0..4095 → stretched logit.
#[inline]
pub fn stretch(p: u16) -> i32 {
    let i = (p as usize).min(4095);
    stretch_table()[i] as i32
}

/// Convert 12-bit p to 16-bit p1 for the arithmetic coder (clamped away from ends).
#[inline]
pub fn p12_to_p16(p: u16) -> u16 {
    let v = ((p as u32 * 65535) / 4095) as u16;
    if v < 1 {
        1
    } else if v > 65534 {
        65534
    } else {
        v
    }
}

/// Convert 16-bit p1 → 12-bit for APM inputs.
#[inline]
pub fn p16_to_p12(p: u16) -> u16 {
    ((p as u32 * 4095) / 65535) as u16
}

/// One-stage adaptive probability map, paq8-style with interpolated stretch bins.
pub struct Apm {
    t: Box<[u16]>,
    n: usize,
    /// Last context base index into `t` (= cxt * 33).
    cbase: usize,
    /// Last bin index 0..31.
    bi: usize,
    /// Last interpolation weight 0..127.
    bw: u32,
    /// Adaptation shift (larger = slower). Default 7 (careful).
    rate_shift: u32,
}

impl Apm {
    pub fn new(n: usize) -> Self {
        let mut t = vec![0u16; n * 33].into_boxed_slice();
        for c in 0..n {
            for i in 0..33 {
                let d = (i as i32 - 16) * 128;
                // Store probability * 16 for finer updates.
                t[c * 33 + i] = (squash(d) as u32 * 16) as u16;
            }
        }
        Self {
            t,
            n,
            cbase: 0,
            bi: 16,
            bw: 0,
            rate_shift: 7,
        }
    }

    pub fn with_rate_shift(mut self, rate_shift: u32) -> Self {
        self.rate_shift = rate_shift.clamp(4, 12);
        self
    }

    /// Predict refined probability (12-bit) given raw `p` (12-bit) and context.
    pub fn predict(&mut self, p: u16, cxt: usize) -> u16 {
        let cxt = cxt % self.n;
        self.cbase = cxt * 33;
        let s = stretch(p) + 2047; // 0..4094
        self.bi = (s >> 7) as usize; // 0..31
        self.bw = (s & 127) as u32;
        let a = self.t[self.cbase + self.bi] as u32;
        let b = self.t[self.cbase + self.bi + 1] as u32;
        let mixed = (a * (128 - self.bw) + b * self.bw) >> 7;
        let mut out = (mixed >> 4) as u16;
        if out > 4095 {
            out = 4095;
        }
        out
    }

    /// Adapt the bins that contributed to the last prediction.
    pub fn update(&mut self, bit: u8) {
        let tgt = if bit != 0 { 65520i32 } else { 0i32 };
        let rs = self.rate_shift;
        for off in 0..2 {
            let idx = self.cbase + self.bi + off;
            if idx >= self.t.len() {
                break;
            }
            let cur = self.t[idx] as i32;
            let next = cur + ((tgt - cur) >> rs);
            self.t[idx] = next.clamp(0, 65520) as u16;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squash_stretch_roundtrip_mid() {
        let mid = squash(0);
        assert!((2000..2200).contains(&mid), "squash(0)={mid}");
        let s = stretch(2048);
        assert!(s.abs() < 40, "stretch(2048)={s}");
    }

    #[test]
    fn apm_near_passthrough_initially() {
        let mut a = Apm::new(8);
        for &p in &[200u16, 1024, 2048, 3500] {
            let out = a.predict(p, 0);
            let diff = (out as i32 - p as i32).abs();
            assert!(diff <= 20, "init passthrough p={p} out={out}");
        }
    }

    #[test]
    fn apm_adapts_toward_ones() {
        let mut a = Apm::new(16);
        let mut p = 2048u16;
        for _ in 0..400 {
            p = a.predict(2048, 3);
            a.update(1);
        }
        assert!(p > 2400, "expected APM to raise p after many ones, got {p}");
    }

    #[test]
    fn apm_deterministic() {
        let mut a = Apm::new(8);
        let mut b = Apm::new(8);
        let bits = [0u8, 1, 1, 0, 1, 0, 0, 1, 1, 1];
        for &bit in &bits {
            assert_eq!(a.predict(1800, 2), b.predict(1800, 2));
            a.update(bit);
            b.update(bit);
        }
    }
}
