//! Tunable compression parameters (compile-time).
//!
//! Tuned by `tune/search.sh` rewriting the `BEGIN TUNABLE` / `END TUNABLE`
//! block. Algorithms stay fixed; only these knobs change.

// === BEGIN TUNABLE ===
/// Order-2 counter rescale when n0+n1 reaches this (adaptation rate).
pub const ORDER2_RESCALE: u16 = 120;

/// Match history buffer size = 1 << BUF_BITS.
pub const BUF_BITS: usize = 25;
/// Match hash table size = 1 << HASH_BITS.
pub const HASH_BITS: usize = 26;
/// Bytes hashed for match lookup.
pub const HASH_ORDER: usize = 6;
/// Minimum match length before ensemble uses match probabilities.
pub const MIN_MIX_LEN: u32 = 6;

/// Match-model confidence (offset from 32768) by match-length bucket:
/// [5..=6, 7..=10, 11..=20, 21..=40, 41+].
pub const MATCH_CONF: [u32; 5] = [32700, 32700, 32700, 32700, 32700];

/// Ensemble mix weight (0..=255) for match vs order-2, same buckets.
pub const MIX_WEIGHT: [u32; 5] = [100, 160, 200, 230, 245];
// === END TUNABLE ===

pub const BUF_SIZE: usize = 1 << BUF_BITS;
pub const BUF_MASK: usize = BUF_SIZE - 1;
pub const HASH_SIZE: usize = 1 << HASH_BITS;
pub const HASH_MASK: usize = HASH_SIZE - 1;

#[inline]
pub fn match_conf(match_len: u32) -> u32 {
    match match_len {
        0..=4 => 0,
        5..=6 => MATCH_CONF[0],
        7..=10 => MATCH_CONF[1],
        11..=20 => MATCH_CONF[2],
        21..=40 => MATCH_CONF[3],
        _ => MATCH_CONF[4],
    }
}

#[inline]
pub fn mix_weight(match_len: u32) -> u32 {
    match match_len {
        0..=4 => 0,
        5..=6 => MIX_WEIGHT[0],
        7..=10 => MIX_WEIGHT[1],
        11..=20 => MIX_WEIGHT[2],
        21..=40 => MIX_WEIGHT[3],
        _ => MIX_WEIGHT[4],
    }
}
