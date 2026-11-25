//! Tunable compression parameters (compile-time).
//!
//! Tuned by `tune/search.sh` rewriting the `BEGIN TUNABLE` / `END TUNABLE`
//! block. Algorithms stay fixed; only these knobs change.

// === BEGIN TUNABLE ===
/// Order-2 counter rescale when n0+n1 reaches this (adaptation rate).
pub const ORDER2_RESCALE: u16 = 250;

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
pub const MIX_WEIGHT: [u32; 5] = [120, 160, 190, 230, 245];

/// When match is inactive: blend weight (0..=255) of XML/wiki structure into the mix.
pub const STRUCT_MIX_WEIGHT: u32 = 24;

/// Blend weight (0..=255) of order-1 into order-2 before match/struct mix.
pub const ORDER1_MIX_WEIGHT: u32 = 8;

/// Hashed order-3 table bits (size = 1 << this).
pub const ORDER3_BITS: usize = 24;
/// Blend weight (0..=255) of hashed order-3 into the CM mix.
pub const ORDER3_MIX_WEIGHT: u32 = 140;

/// Hashed order-4 table bits.
pub const ORDER4_BITS: usize = 25;
/// Blend weight (0..=255) of hashed order-4 into the CM mix.
pub const ORDER4_MIX_WEIGHT: u32 = 88;

/// Hashed order-5 table bits.
pub const ORDER5_BITS: usize = 24;
/// Blend weight (0..=255) of hashed order-5 into the CM mix.
pub const ORDER5_MIX_WEIGHT: u32 = 88;

/// Hashed order-6 table bits.
pub const ORDER6_BITS: usize = 24;
/// Blend weight (0..=255) of hashed order-6 into the CM mix.
pub const ORDER6_MIX_WEIGHT: u32 = 32;

/// Hashed order-7 table bits.
pub const ORDER7_BITS: usize = 23;
/// Blend weight (0..=255) of hashed order-7 into the CM mix.
pub const ORDER7_MIX_WEIGHT: u32 = 16;

/// SSE blend weight (0..=255) of secondary estimate into final p.
pub const SSE_MIX_WEIGHT: u32 = 224;

/// Word-model hash table bits.
pub const WORD_BITS: usize = 23;
/// Blend weight (0..=255) of word model into CM mix (before match/SSE).
pub const WORD_MIX_WEIGHT: u32 = 64;

/// Second SSE (byte-class context) blend weight.
pub const SSE2_MIX_WEIGHT: u32 = 112;

/// Blend weight (0..=255) of order-0 into CM mix.
pub const ORDER0_MIX_WEIGHT: u32 = 80;

/// Indirect / record model hash table bits.
pub const INDIRECT_BITS: usize = 24;
/// Blend weight (0..=255) of indirect model into CM mix.
pub const INDIRECT_MIX_WEIGHT: u32 = 28;

/// Blend weight (0..=255) of match×indirect cross model into CM mix.
pub const MATCH_IND_CROSS_WEIGHT: u32 = 10;

/// Third-stage APM blend weight (0..=255) into final p after SSE2.
pub const APM_MIX_WEIGHT: u32 = 224;
/// APM adaptation shift (larger = slower). Careful default.
pub const APM_RATE_SHIFT: u32 = 7;
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
