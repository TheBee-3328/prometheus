//! Debug ablation + instrumentation for PROMETHEUS.
//! NOT shipped in SCORE path; build with: cargo build --release -p prometheus-diag

use prometheus_core::params::{
    match_conf, mix_weight, HASH_BITS, HASH_ORDER, MIN_MIX_LEN, STRUCT_MIX_WEIGHT,
};
use prometheus_core::{
    encode, ArithmeticEncoder, MatchModel, Model, Order2MatchModel, Order2Model, SparseModel,
    MAGIC,
};
use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::process;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Full,
    NoPre,
    NoMatch,
    NoStruct,
    Order2Only,
    MatchOnly,
}

fn parse_mode(s: &str) -> Option<Mode> {
    match s {
        "full" => Some(Mode::Full),
        "no-pre" => Some(Mode::NoPre),
        "no-match" => Some(Mode::NoMatch),
        "no-struct" => Some(Mode::NoStruct),
        "order2-only" => Some(Mode::Order2Only),
        "match-only" => Some(Mode::MatchOnly),
        _ => None,
    }
}

fn usage() -> ! {
    eprintln!(
        "usage: prometheus-diag <input> <output> --mode <full|no-pre|no-match|no-struct|order2-only|match-only> [--stats]"
    );
    process::exit(2);
}

fn mix_w(p_a: u16, p_b: u16, w: u32) -> u16 {
    let p = (p_a as u32 * (256 - w) + p_b as u32 * w) / 256;
    if p < 1 {
        1
    } else if p > 65534 {
        65534
    } else {
        p as u16
    }
}

/// Cost in milli-bits for one bit under prediction p1 (fixed-point, no FP).
/// approx -log2(p_actual) * 1000 using a small LUT + linear interp on stretch scale.
fn bit_cost_mbits(bit: u8, p1: u16) -> u32 {
    let p = if bit == 0 {
        65536u32.saturating_sub(p1 as u32)
    } else {
        p1 as u32
    };
    let p = p.max(1);
    // cost ≈ -log2(p/65536) * 1000 = (16 - log2(p)) * 1000
    // use integer log2 + remainder refinement
    let lz = p.leading_zeros();
    let log2_floor = 31 - lz; // floor(log2(p))
    let frac_num = p - (1u32 << log2_floor);
    // frac ≈ frac_num / 2^log2_floor in 0..1; contribute ~frac*1000/ln(2)≈1443*frac/1000 wait
    // -log2(p) = -log2_floor - log2(1+f) ≈ -log2_floor - f/(ln2) with f=frac_num/2^k
    // log2(1+f)≈ f * 1.4427 ≈ f * 1443/1000
    let frac_x1000 = if log2_floor == 0 {
        0
    } else {
        (frac_num * 1000) / (1u32 << log2_floor)
    };
    let log2_x1000 = log2_floor * 1000 + (frac_x1000 * 1443) / 1000;
    // -log2(p/65536)*1000 = (16*1000 - log2(p)*1000)
    16000u32.saturating_sub(log2_x1000)
}

struct Stats {
    bits: u64,
    cost_mbits: u64,
    /// by byte class of the *completed* previous byte context approximation:
    /// accumulate cost into class of last finished byte
    class_cost: [u64; 9],
    class_bits: [u64; 9],
    match_active_bits: u64,
    match_len_sum: u64,
    match_hit_bytes: u64,
    match_bytes: u64,
    p_min_pin: u64,
    p_max_pin: u64,
    hash_writes: u64,
}

impl Stats {
    fn new() -> Self {
        Self {
            bits: 0,
            cost_mbits: 0,
            class_cost: [0; 9],
            class_bits: [0; 9],
            match_active_bits: 0,
            match_len_sum: 0,
            match_hit_bytes: 0,
            match_bytes: 0,
            p_min_pin: 0,
            p_max_pin: 0,
            hash_writes: 0,
        }
    }
}

fn byte_class(b: u8) -> usize {
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

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let mut stats_flag = false;
    if let Some(i) = args.iter().position(|a| a == "--stats") {
        stats_flag = true;
        args.remove(i);
    }
    let mode_idx = args.iter().position(|a| a == "--mode");
    let mode = if let Some(i) = mode_idx {
        if i + 1 >= args.len() {
            usage();
        }
        let m = parse_mode(&args[i + 1]).unwrap_or_else(|| usage());
        args.drain(i..=i + 1);
        m
    } else {
        Mode::Full
    };
    if args.len() != 2 {
        usage();
    }
    let input = &args[0];
    let output = &args[1];

    if let Err(e) = run(input, output, mode, stats_flag) {
        eprintln!("prometheus-diag: {e}");
        process::exit(1);
    }
}

fn run(input: &str, output: &str, mode: Mode, stats_flag: bool) -> io::Result<()> {
    let mut raw = Vec::new();
    File::open(input)?.read_to_end(&mut raw)?;
    let use_pre = mode != Mode::NoPre;
    let data = if use_pre { encode(&raw) } else { raw };

    let out = BufWriter::new(File::create(output)?);
    let mut out = out;
    out.write_all(MAGIC)?;
    out.write_all(&(data.len() as u64).to_le_bytes())?;

    let mut finished = match mode {
        Mode::Full | Mode::NoPre => compress_full(&data, out, stats_flag)?,
        Mode::NoMatch => compress_no_match(&data, out, stats_flag)?,
        Mode::NoStruct => compress_no_struct(&data, out, stats_flag)?,
        Mode::Order2Only => compress_order2(&data, out, stats_flag)?,
        Mode::MatchOnly => compress_match_only(&data, out, stats_flag)?,
    };
    finished.flush()?;
    drop(finished);

    let bytes = std::fs::metadata(output)?.len();
    println!(
        "MODE={:?} PRE={} INPUT={} PRE_LEN={} COMPRESSED={} HASH_BITS={} HASH_ORDER={} MIN_MIX_LEN={} STRUCT_MIX={}",
        mode,
        use_pre as u8,
        std::fs::metadata(input)?.len(),
        data.len(),
        bytes,
        HASH_BITS,
        HASH_ORDER,
        MIN_MIX_LEN,
        STRUCT_MIX_WEIGHT
    );
    Ok(())
}

fn compress_full<W: Write>(
    data: &[u8],
    out: W,
    stats_flag: bool,
) -> io::Result<W> {
    let mut model = Order2MatchModel::new();
    let mut match_m = MatchModel::new(); // parallel for stats only
    let mut enc = ArithmeticEncoder::new(out);
    let mut st = Stats::new();
    let mut last_class = 8usize;
    let mut bit_in_byte = 0u8;

    for &byte in data {
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1;
            let p = model.predict();
            if stats_flag {
                record_bit(&mut st, bit, p, match_m.match_len(), last_class);
            }
            enc.encode_bit(bit, p)?;
            model.update(bit);
            if stats_flag {
                match_m.update(bit);
            }
            bit_in_byte += 1;
            if bit_in_byte == 8 {
                bit_in_byte = 0;
                last_class = byte_class(byte);
                if stats_flag {
                    st.match_bytes += 1;
                    if match_m.match_len() >= MIN_MIX_LEN {
                        st.match_hit_bytes += 1;
                    }
                }
            }
        }
    }
    if stats_flag {
        print_stats(&st);
    }
    enc.finish()
}

fn compress_no_match<W: Write>(data: &[u8], out: W, stats_flag: bool) -> io::Result<W> {
    let mut o2 = Order2Model::new();
    let mut sparse = SparseModel::new();
    let mut enc = ArithmeticEncoder::new(out);
    let mut st = Stats::new();
    let mut last_class = 8usize;

    for &byte in data {
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1;
            let p0 = o2.predict();
            let px = sparse.predict();
            let p = mix_w(p0, px, STRUCT_MIX_WEIGHT);
            if stats_flag {
                record_bit(&mut st, bit, p, 0, last_class);
            }
            enc.encode_bit(bit, p)?;
            o2.update(bit);
            sparse.update(bit);
        }
        last_class = byte_class(byte);
    }
    if stats_flag {
        print_stats(&st);
    }
    enc.finish()
}

fn compress_no_struct<W: Write>(data: &[u8], out: W, stats_flag: bool) -> io::Result<W> {
    let mut o2 = Order2Model::new();
    let mut match_m = MatchModel::new();
    let mut enc = ArithmeticEncoder::new(out);
    let mut st = Stats::new();
    let mut last_class = 8usize;

    for &byte in data {
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1;
            let p0 = o2.predict();
            let len = match_m.match_len();
            let p = if len >= MIN_MIX_LEN {
                mix_w(p0, match_m.predict(), mix_weight(len))
            } else {
                p0
            };
            if stats_flag {
                record_bit(&mut st, bit, p, len, last_class);
            }
            enc.encode_bit(bit, p)?;
            o2.update(bit);
            match_m.update(bit);
        }
        last_class = byte_class(byte);
        st.match_bytes += 1;
        if match_m.match_len() >= MIN_MIX_LEN {
            st.match_hit_bytes += 1;
        }
    }
    if stats_flag {
        print_stats(&st);
    }
    enc.finish()
}

fn compress_order2<W: Write>(data: &[u8], out: W, stats_flag: bool) -> io::Result<W> {
    let mut o2 = Order2Model::new();
    let mut enc = ArithmeticEncoder::new(out);
    let mut st = Stats::new();
    let mut last_class = 8usize;

    for &byte in data {
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1;
            let p = o2.predict();
            if stats_flag {
                record_bit(&mut st, bit, p, 0, last_class);
            }
            enc.encode_bit(bit, p)?;
            o2.update(bit);
        }
        last_class = byte_class(byte);
    }
    if stats_flag {
        print_stats(&st);
    }
    enc.finish()
}

fn compress_match_only<W: Write>(data: &[u8], out: W, stats_flag: bool) -> io::Result<W> {
    let mut match_m = MatchModel::new();
    let mut enc = ArithmeticEncoder::new(out);
    let mut st = Stats::new();
    let mut last_class = 8usize;

    for &byte in data {
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1;
            let len = match_m.match_len();
            // When inactive, flat 1/2; when active use match predict
            let p = if len >= MIN_MIX_LEN {
                match_m.predict()
            } else {
                32768
            };
            if stats_flag {
                record_bit(&mut st, bit, p, len, last_class);
            }
            enc.encode_bit(bit, p)?;
            match_m.update(bit);
        }
        last_class = byte_class(byte);
        st.match_bytes += 1;
        if match_m.match_len() >= MIN_MIX_LEN {
            st.match_hit_bytes += 1;
        }
    }
    if stats_flag {
        print_stats(&st);
    }
    enc.finish()
}

fn record_bit(st: &mut Stats, bit: u8, p: u16, match_len: u32, class: usize) {
    st.bits += 1;
    let c = bit_cost_mbits(bit, p);
    st.cost_mbits += c as u64;
    st.class_cost[class] += c as u64;
    st.class_bits[class] += 1;
    if match_len >= MIN_MIX_LEN {
        st.match_active_bits += 1;
        st.match_len_sum += match_len as u64;
    }
    if p <= 1 {
        st.p_min_pin += 1;
    }
    if p >= 65534 {
        st.p_max_pin += 1;
    }
    let _ = match_conf(match_len); // keep params linked
}

fn print_stats(st: &Stats) {
    let avg_bpb = if st.bits > 0 {
        (st.cost_mbits * 1000) / st.bits // micro-bits per bit → report as mbits/bit
    } else {
        0
    };
    println!(
        "STATS bits={} cost_mbits={} avg_mbits_per_bit={} match_active_bits={} match_active_pct={} avg_match_len_x100={} match_hit_bytes={} match_hit_pct={} p_min_pin={} p_max_pin={} hash_bits={}",
        st.bits,
        st.cost_mbits,
        avg_bpb,
        st.match_active_bits,
        if st.bits > 0 {
            st.match_active_bits * 10000 / st.bits
        } else {
            0
        },
        if st.match_active_bits > 0 {
            st.match_len_sum * 100 / st.match_active_bits
        } else {
            0
        },
        st.match_hit_bytes,
        if st.match_bytes > 0 {
            st.match_hit_bytes * 10000 / st.match_bytes
        } else {
            0
        },
        st.p_min_pin,
        st.p_max_pin,
        HASH_BITS
    );
    let names = [
        "ws", "alpha", "digit", "lt", "gt", "entity", "bracket", "hibyte", "other",
    ];
    for (i, name) in names.iter().enumerate() {
        if st.class_bits[i] == 0 {
            continue;
        }
        let avg = st.class_cost[i] * 1000 / st.class_bits[i];
        println!(
            "CLASS {} bits={} cost_mbits={} avg_mbits_per_bit={}",
            name, st.class_bits[i], st.class_cost[i], avg
        );
    }
}
