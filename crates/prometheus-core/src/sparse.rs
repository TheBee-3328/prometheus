//! XML/wiki structure context model.
//!
//! Ensembled with order-2 only when the match model is inactive — never
//! replaces match. Integer / fixed-point only; no HashMap.

use crate::params::ORDER2_RESCALE;

const N_CLASS: usize = 9;
const STRUCT_STATES: usize = 8;
/// index = state * N_CLASS * 256 + class * 256 + bit_cxt
const STRUCT_SIZE: usize = STRUCT_STATES * N_CLASS * 256;

/// Structure-state bit predictor for preprocessed enwik streams.
pub struct SparseModel {
    bit_cxt: u16,
    last_class: u8,
    /// 0=text 1=after_lt 2=tag 3=entity 4=after_dict 5=bang 6=attr 7=bracket
    struct_st: u8,
    sn0: Box<[u8]>,
    sn1: Box<[u8]>,
}

impl SparseModel {
    pub fn new() -> Self {
        Self {
            bit_cxt: 1,
            last_class: 0,
            struct_st: 0,
            sn0: vec![0u8; STRUCT_SIZE].into_boxed_slice(),
            sn1: vec![0u8; STRUCT_SIZE].into_boxed_slice(),
        }
    }

    #[inline]
    fn byte_class(b: u8) -> u8 {
        match b {
            b' ' | b'\n' | b'\t' | b'\r' => 0,
            b'a'..=b'z' | b'A'..=b'Z' => 1,
            b'0'..=b'9' => 2,
            b'<' => 3,
            b'>' => 4,
            b'&' | b';' => 5,
            b'[' | b']' | b'{' | b'}' => 6,
            0x80..=0xfe => 7,
            _ => 8,
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

    #[inline]
    fn struct_index(&self) -> usize {
        (self.struct_st as usize) * N_CLASS * 256
            + (self.last_class as usize) * 256
            + (self.bit_cxt as usize)
    }

    #[inline]
    pub fn predict_struct(&self) -> u16 {
        let i = self.struct_index();
        Self::prob(self.sn0[i], self.sn1[i])
    }

    #[inline]
    fn learn(n0: &mut u8, n1: &mut u8, bit: u8) {
        if bit == 0 {
            *n0 = n0.saturating_add(1);
        } else {
            *n1 = n1.saturating_add(1);
        }
        if (*n0 as u16) + (*n1 as u16) >= ORDER2_RESCALE {
            *n0 >>= 1;
            *n1 >>= 1;
        }
    }

    fn update_struct_state(&mut self, byte: u8) {
        if byte >= 0x80 && byte != 0xff {
            self.struct_st = 4;
            return;
        }
        match self.struct_st {
            1 => {
                self.struct_st = if byte == b'!' {
                    5
                } else if byte == b'/'
                    || (byte >= b'a' && byte <= b'z')
                    || (byte >= b'A' && byte <= b'Z')
                {
                    2
                } else {
                    0
                };
            }
            2 => {
                if byte == b'>' {
                    self.struct_st = 0;
                } else if byte == b'"' || byte == b'\'' {
                    self.struct_st = 6;
                }
            }
            6 => {
                if byte == b'"' || byte == b'\'' {
                    self.struct_st = 2;
                }
            }
            3 => {
                if byte == b';' || byte <= b' ' {
                    self.struct_st = 0;
                }
            }
            5 => {
                if byte == b'>' {
                    self.struct_st = 0;
                }
            }
            7 => {
                if byte == b']' || byte == b'}' {
                    self.struct_st = 0;
                } else if byte == b'<' {
                    self.struct_st = 1;
                }
            }
            4 => {
                if byte == b'<' {
                    self.struct_st = 1;
                } else if byte == b'&' {
                    self.struct_st = 3;
                } else if byte == b'[' || byte == b'{' {
                    self.struct_st = 7;
                } else if byte >= 0x80 && byte != 0xff {
                    self.struct_st = 4;
                } else {
                    self.struct_st = 0;
                }
            }
            _ => {
                if byte == b'<' {
                    self.struct_st = 1;
                } else if byte == b'&' {
                    self.struct_st = 3;
                } else if byte == b'[' || byte == b'{' {
                    self.struct_st = 7;
                }
            }
        }
    }

    fn finish_byte(&mut self, byte: u8) {
        self.last_class = Self::byte_class(byte);
        self.update_struct_state(byte);
        self.bit_cxt = 1;
    }
}

impl Default for SparseModel {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::model::Model for SparseModel {
    #[inline]
    fn predict(&mut self) -> u16 {
        self.predict_struct()
    }

    #[inline]
    fn update(&mut self, bit: u8) {
        let i = self.struct_index();
        let mut a = self.sn0[i];
        let mut b = self.sn1[i];
        Self::learn(&mut a, &mut b, bit);
        self.sn0[i] = a;
        self.sn1[i] = b;

        self.bit_cxt = (self.bit_cxt << 1) | (bit as u16);
        if self.bit_cxt >= 256 {
            let byte = (self.bit_cxt & 0xff) as u8;
            self.finish_byte(byte);
        }
    }
}
