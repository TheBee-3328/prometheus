//! Prose / alpha residual contexts beyond [`crate::word::WordModel`].
//!
//! Integer only. Mode selected by [`crate::params::PROSE_MODE`] for slice hyps.
//! Predictions are hit-gated (checksum miss / cold slot → no mix).

use crate::model::Model;
use crate::params::{ORDER2_RESCALE, PROSE_BITS, PROSE_MODE, PROSE_SM};
use crate::preprocessor::capitals::is_cap_marker;
use crate::statemap::{sm_init, sm_predict, sm_update};

#[inline]
fn pack_counts(n0: u8, n1: u8) -> u16 {
    (n0 as u16) | ((n1 as u16) << 8)
}

#[inline]
fn unpack_counts(c: u16) -> (u8, u8) {
    (c as u8, (c >> 8) as u8)
}

/// Specialized alphabetic-run predictor (position / prev-word / skip / cap).
pub struct ProseModel {
    cur_hash: u32,
    prev_hash: u32,
    wpos: u8,
    last_a: u8,
    last_a2: u8,
    last_a3: u8,
    after_cap: bool,
    bit_cxt: u16,
    /// Packed n0/n1 when not using StateMap.
    counts: Box<[u16]>,
    cells: Box<[u32]>,
    chk: Box<[u8]>,
    mask: usize,
    last_i: usize,
    last_cs: u8,
    have_slot: bool,
}

impl ProseModel {
    pub fn new() -> Self {
        let size = 1usize << PROSE_BITS;
        let use_sm = PROSE_SM != 0;
        Self {
            cur_hash: 0,
            prev_hash: 0,
            wpos: 0,
            last_a: 0,
            last_a2: 0,
            last_a3: 0,
            after_cap: false,
            bit_cxt: 1,
            counts: if use_sm {
                Vec::new().into_boxed_slice()
            } else {
                vec![0u16; size].into_boxed_slice()
            },
            cells: if use_sm {
                vec![sm_init(); size].into_boxed_slice()
            } else {
                Vec::new().into_boxed_slice()
            },
            chk: vec![0u8; size].into_boxed_slice(),
            mask: size - 1,
            last_i: 0,
            last_cs: 0,
            have_slot: false,
        }
    }

    #[inline]
    fn is_letter(b: u8) -> bool {
        matches!(b, b'a'..=b'z' | b'A'..=b'Z')
    }

    #[inline]
    fn to_lc(b: u8) -> u8 {
        if (b'A'..=b'Z').contains(&b) {
            b + 32
        } else {
            b
        }
    }

    #[inline]
    fn context_hash(&self) -> u32 {
        let mut h: u32 = 2166136261;
        h ^= PROSE_MODE as u32;
        h = h.wrapping_mul(16777619);
        h ^= self.bit_cxt as u32;
        h = h.wrapping_mul(16777619);
        match PROSE_MODE {
            1 => {
                h ^= self.wpos as u32;
                h = h.wrapping_mul(16777619);
            }
            2 => {
                h ^= self.prev_hash;
                h = h.wrapping_mul(16777619);
                h ^= self.cur_hash;
                h = h.wrapping_mul(16777619);
            }
            3 => {
                h ^= self.wpos as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.last_a as u32;
                h = h.wrapping_mul(16777619);
            }
            4 => {
                h ^= self.last_a as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.last_a2 as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.last_a3 as u32;
                h = h.wrapping_mul(16777619);
            }
            5 => {
                h ^= self.prev_hash;
                h = h.wrapping_mul(16777619);
                h ^= self.wpos as u32;
                h = h.wrapping_mul(16777619);
            }
            6 => {
                h ^= self.after_cap as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.wpos as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.last_a as u32;
                h = h.wrapping_mul(16777619);
            }
            7 => {
                h ^= self.prev_hash;
                h = h.wrapping_mul(16777619);
                h ^= self.cur_hash;
                h = h.wrapping_mul(16777619);
                h ^= self.wpos as u32;
                h = h.wrapping_mul(16777619);
            }
            8 => {
                h ^= self.after_cap as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.prev_hash;
                h = h.wrapping_mul(16777619);
                h ^= self.cur_hash;
                h = h.wrapping_mul(16777619);
            }
            9 => {
                h ^= self.prev_hash;
                h = h.wrapping_mul(16777619);
                h ^= (self.wpos as u32) & 3;
                h = h.wrapping_mul(16777619);
            }
            10 => {
                h ^= self.after_cap as u32;
                h = h.wrapping_mul(16777619);
                h ^= self.wpos as u32;
                h = h.wrapping_mul(16777619);
            }
            _ => {}
        }
        h
    }

    #[inline]
    fn index_cs(&self) -> (usize, u8) {
        let h = self.context_hash();
        let idx = (h as usize) & self.mask;
        let mut cs = (h.wrapping_mul(0x85ebca6b) >> 24) as u8;
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

    #[inline]
    fn active(&self) -> bool {
        match PROSE_MODE {
            0 => false,
            1 | 3 | 4 | 6 | 10 => self.wpos > 0 || self.last_a != 0 || self.after_cap,
            2 | 5 | 7 | 8 | 9 => self.wpos > 0 || self.prev_hash != 0 || self.cur_hash != 0,
            _ => false,
        }
    }

    /// Hit-gated prediction: `None` = do not mix (miss / cold / inactive).
    #[inline]
    pub fn predict_hit(&mut self) -> Option<u16> {
        if !self.active() {
            self.have_slot = false;
            return None;
        }
        let (i, cs) = self.index_cs();
        self.last_i = i;
        self.last_cs = cs;
        self.have_slot = true;
        if self.chk[i] != cs {
            return None;
        }
        if PROSE_SM != 0 {
            let n = self.cells[i] & 0xff;
            if n < 1 {
                return None;
            }
            Some(sm_predict(self.cells[i]))
        } else {
            let (n0, n1) = unpack_counts(self.counts[i]);
            if n0 == 0 && n1 == 0 {
                return None;
            }
            Some(Self::prob(n0, n1))
        }
    }
}

impl Default for ProseModel {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for ProseModel {
    #[inline]
    fn predict(&mut self) -> u16 {
        self.predict_hit().unwrap_or(32768)
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        if self.active() {
            let (i, cs) = if self.have_slot {
                (self.last_i, self.last_cs)
            } else {
                self.index_cs()
            };
            if self.chk[i] != cs {
                self.chk[i] = cs;
                if PROSE_SM != 0 {
                    self.cells[i] = sm_init();
                } else {
                    self.counts[i] = 0;
                }
            }
            if PROSE_SM != 0 {
                self.cells[i] = sm_update(self.cells[i], bit);
            } else {
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
            }
        }
        self.have_slot = false;
        self.bit_cxt = (self.bit_cxt << 1) | (bit as u16);
        if self.bit_cxt >= 256 {
            let byte = (self.bit_cxt & 0xff) as u8;
            if Self::is_letter(byte) {
                let lc = Self::to_lc(byte);
                self.last_a3 = self.last_a2;
                self.last_a2 = self.last_a;
                self.last_a = lc;
                if self.wpos == 0 {
                    self.cur_hash = lc as u32;
                    self.wpos = 1;
                } else {
                    self.cur_hash = self.cur_hash.wrapping_mul(263).wrapping_add(lc as u32);
                    if self.wpos < 31 {
                        self.wpos += 1;
                    }
                }
                self.after_cap = false;
            } else {
                if self.wpos > 0 {
                    self.prev_hash = self.cur_hash;
                    self.cur_hash = 0;
                    self.wpos = 0;
                }
                self.after_cap = is_cap_marker(byte);
            }
            self.bit_cxt = 1;
        }
    }
}
