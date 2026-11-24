//! Paq-style count-dependent StateMap (dual-rate via 3/(n+4) step).
//!
//! Each cell packs probability (hi16) and visit count (lo8). Early visits
//! adapt fast; saturated counts slow down — not blanket u16 n0/n1 counters.

/// Packed StateMap cell: hi16 = p (1..=65534), lo8 = n (0..=255).
#[inline]
pub fn sm_init() -> u32 {
    (32768u32 << 16) | 0
}

#[inline]
pub fn sm_predict(cell: u32) -> u16 {
    let p = (cell >> 16) as u16;
    if p < 1 {
        1
    } else if p > 65534 {
        65534
    } else {
        p
    }
}

#[inline]
pub fn sm_update(cell: u32, bit: u8) -> u32 {
    let mut p = (cell >> 16) as i32;
    let n = (cell & 0xff) as i32;
    let tgt = if bit != 0 { 65535i32 } else { 0i32 };
    // paq-ish dual-rate: step ≈ 3/(n+4) of the residual
    p += ((tgt - p) * 3) / (n + 4);
    let n2 = if n < 255 { (n + 1) as u32 } else { 255 };
    ((p.clamp(1, 65534) as u32) << 16) | n2
}
