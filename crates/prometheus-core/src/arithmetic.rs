//! 32-bit binary arithmetic coder with 16-bit probabilities.
//! Witten–Neal–Cleary style with outstanding-bit underflow handling.

use std::io::{self, Read, Write};

const HALF: u32 = 1 << 31;
const FIRST_QTR: u32 = 1 << 30;
const THIRD_QTR: u32 = 3 << 30;

struct BitWriter<W: Write> {
    buf: u8,
    nbits: u8,
    out: W,
}

impl<W: Write> BitWriter<W> {
    fn new(out: W) -> Self {
        Self {
            buf: 0,
            nbits: 0,
            out,
        }
    }

    fn put_bit(&mut self, bit: u8) -> io::Result<()> {
        self.buf = (self.buf << 1) | (bit & 1);
        self.nbits += 1;
        if self.nbits == 8 {
            self.out.write_all(&[self.buf])?;
            self.buf = 0;
            self.nbits = 0;
        }
        Ok(())
    }

    fn finish(mut self) -> io::Result<W> {
        if self.nbits > 0 {
            self.buf <<= 8 - self.nbits;
            self.out.write_all(&[self.buf])?;
        }
        self.out.flush()?;
        Ok(self.out)
    }
}

struct BitReader<R: Read> {
    buf: u8,
    nbits: u8,
    input: R,
}

impl<R: Read> BitReader<R> {
    fn new(input: R) -> Self {
        Self {
            buf: 0,
            nbits: 0,
            input,
        }
    }

    fn get_bit(&mut self) -> io::Result<u8> {
        if self.nbits == 0 {
            let mut b = [0u8; 1];
            if self.input.read(&mut b)? == 0 {
                // Past end: feed zeros (trailing padding from encoder finish).
                return Ok(0);
            }
            self.buf = b[0];
            self.nbits = 8;
        }
        self.nbits -= 1;
        Ok((self.buf >> self.nbits) & 1)
    }
}

/// Encodes bits with probability `p1` = P(bit=1) in 1..=65534.
pub struct ArithmeticEncoder<W: Write> {
    low: u32,
    high: u32,
    outstanding: u32,
    bits: BitWriter<W>,
}

impl<W: Write> ArithmeticEncoder<W> {
    pub fn new(out: W) -> Self {
        Self {
            low: 0,
            high: 0xffff_ffff,
            outstanding: 0,
            bits: BitWriter::new(out),
        }
    }

    fn put_bit_follow(&mut self, bit: u8) -> io::Result<()> {
        self.bits.put_bit(bit)?;
        let follow = 1 - bit;
        while self.outstanding > 0 {
            self.bits.put_bit(follow)?;
            self.outstanding -= 1;
        }
        Ok(())
    }

    fn renorm(&mut self) -> io::Result<()> {
        loop {
            if self.high < HALF {
                self.put_bit_follow(0)?;
                self.low <<= 1;
                self.high = (self.high << 1) | 1;
            } else if self.low >= HALF {
                self.put_bit_follow(1)?;
                self.low = (self.low - HALF) << 1;
                self.high = ((self.high - HALF) << 1) | 1;
            } else if self.low >= FIRST_QTR && self.high < THIRD_QTR {
                self.outstanding += 1;
                self.low = (self.low - FIRST_QTR) << 1;
                self.high = ((self.high - FIRST_QTR) << 1) | 1;
            } else {
                break;
            }
        }
        Ok(())
    }

    pub fn encode_bit(&mut self, bit: u8, p1: u16) -> io::Result<()> {
        debug_assert!((1..=65534).contains(&p1));
        let range = (self.high - self.low) as u64 + 1;
        // one_len = |interval for bit=1|; 0 gets the lower part, 1 the upper.
        let mut one_len = ((range * p1 as u64) >> 16) as u32;
        if one_len == 0 {
            one_len = 1;
        }
        if (one_len as u64) >= range {
            one_len = (range - 1) as u32;
        }
        let split = self.low + (range - one_len as u64) as u32;
        if bit != 0 {
            self.low = split;
        } else {
            self.high = split - 1;
        }
        self.renorm()
    }

    pub fn finish(mut self) -> io::Result<W> {
        // Force the interval to a unique stream suffix.
        self.outstanding += 1;
        if self.low < FIRST_QTR {
            self.put_bit_follow(0)?;
        } else {
            self.put_bit_follow(1)?;
        }
        self.bits.finish()
    }
}

/// Decodes bits with matching `p1` schedule.
pub struct ArithmeticDecoder<R: Read> {
    low: u32,
    high: u32,
    code: u32,
    bits: BitReader<R>,
}

impl<R: Read> ArithmeticDecoder<R> {
    pub fn new(input: R) -> io::Result<Self> {
        let mut bits = BitReader::new(input);
        let mut code = 0u32;
        for _ in 0..32 {
            code = (code << 1) | (bits.get_bit()? as u32);
        }
        Ok(Self {
            low: 0,
            high: 0xffff_ffff,
            code,
            bits,
        })
    }

    fn renorm(&mut self) -> io::Result<()> {
        loop {
            if self.high < HALF {
                self.low <<= 1;
                self.high = (self.high << 1) | 1;
                self.code = (self.code << 1) | (self.bits.get_bit()? as u32);
            } else if self.low >= HALF {
                self.low = (self.low - HALF) << 1;
                self.high = ((self.high - HALF) << 1) | 1;
                self.code = ((self.code - HALF) << 1) | (self.bits.get_bit()? as u32);
            } else if self.low >= FIRST_QTR && self.high < THIRD_QTR {
                self.low = (self.low - FIRST_QTR) << 1;
                self.high = ((self.high - FIRST_QTR) << 1) | 1;
                self.code = ((self.code - FIRST_QTR) << 1) | (self.bits.get_bit()? as u32);
            } else {
                break;
            }
        }
        Ok(())
    }

    pub fn decode_bit(&mut self, p1: u16) -> io::Result<u8> {
        debug_assert!((1..=65534).contains(&p1));
        let range = (self.high - self.low) as u64 + 1;
        let mut one_len = ((range * p1 as u64) >> 16) as u32;
        if one_len == 0 {
            one_len = 1;
        }
        if (one_len as u64) >= range {
            one_len = (range - 1) as u32;
        }
        let split = self.low + (range - one_len as u64) as u32;
        let bit = if self.code >= split { 1u8 } else { 0u8 };
        if bit != 0 {
            self.low = split;
        } else {
            self.high = split - 1;
        }
        self.renorm()?;
        Ok(bit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn roundtrip(bits: &[u8], probs: &[u16]) {
        assert_eq!(bits.len(), probs.len());
        let mut buf = Vec::new();
        {
            let mut enc = ArithmeticEncoder::new(&mut buf);
            for (&bit, &p) in bits.iter().zip(probs.iter()) {
                enc.encode_bit(bit, p).unwrap();
            }
            enc.finish().unwrap();
        }
        let mut dec = ArithmeticDecoder::new(Cursor::new(&buf)).unwrap();
        for (i, (&bit, &p)) in bits.iter().zip(probs.iter()).enumerate() {
            let got = dec.decode_bit(p).unwrap();
            assert_eq!(got, bit, "bit mismatch at {i}");
        }
    }

    #[test]
    fn encode_decode_symmetry_uniform() {
        let bits: Vec<u8> = (0..1000).map(|i| (i % 3 == 0) as u8).collect();
        let probs = vec![32768u16; bits.len()];
        roundtrip(&bits, &probs);
    }

    #[test]
    fn encode_decode_symmetry_biased() {
        let bits: Vec<u8> = (0..2048).map(|i| ((i * 7) % 11 < 3) as u8).collect();
        let probs: Vec<u16> = bits
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let p = 1000 + ((i * 17) % 60000) as u16;
                p.clamp(1, 65534)
            })
            .collect();
        roundtrip(&bits, &probs);
    }

    #[test]
    fn encode_decode_all_zeros() {
        let bits = vec![0u8; 512];
        let probs = vec![1u16; 512];
        roundtrip(&bits, &probs);
    }

    #[test]
    fn encode_decode_all_ones() {
        let bits = vec![1u8; 512];
        let probs = vec![65534u16; 512];
        roundtrip(&bits, &probs);
    }

    #[test]
    fn encode_decode_alternating() {
        let bits: Vec<u8> = (0..4096).map(|i| (i & 1) as u8).collect();
        let probs = vec![32768u16; bits.len()];
        roundtrip(&bits, &probs);
    }

    #[test]
    fn encode_decode_long_mixed() {
        let bits: Vec<u8> = (0u32..50_000)
            .map(|i| ((i.wrapping_mul(1103515245).wrapping_add(12345) >> 16) & 1) as u8)
            .collect();
        let probs: Vec<u16> = (0..bits.len())
            .map(|i| (1 + (i * 13) % 65534) as u16)
            .collect();
        roundtrip(&bits, &probs);
    }
}
