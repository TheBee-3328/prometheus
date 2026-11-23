//! Decompressor: multi-order (0–6) CM + arithmetic coding (lockstep with compress).

use prometheus_core::{ArithmeticDecoder, Model, OrdersModel, MAGIC};
use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::process;

fn main() {
    let mut args = env::args().skip(1);
    let input = args.next().unwrap_or_else(|| usage());
    let output = args.next().unwrap_or_else(|| usage());
    if args.next().is_some() {
        usage();
    }

    if let Err(e) = decompress(&input, &output) {
        eprintln!("prometheus-decompress: {e}");
        process::exit(1);
    }
}

fn usage() -> ! {
    eprintln!("usage: prometheus-decompress <input> <output>");
    process::exit(2);
}

fn decompress(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut input = BufReader::new(File::open(input_path)?);
    let mut header = [0u8; 12];
    input.read_exact(&mut header)?;
    if &header[0..4] != MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "bad magic"));
    }
    let mut len_bytes = [0u8; 8];
    len_bytes.copy_from_slice(&header[4..12]);
    let declared = u64::from_le_bytes(len_bytes) as usize;

    let mut model = OrdersModel::new();
    let mut dec = ArithmeticDecoder::new(input)?;
    let mut out = BufWriter::new(File::create(output_path)?);
    let mut buf = vec![0u8; declared.min(1 << 20)];
    let mut written = 0usize;

    while written < declared {
        let chunk = (declared - written).min(buf.len());
        for b in buf.iter_mut().take(chunk) {
            let mut byte = 0u8;
            for _ in 0..8 {
                let p = model.predict();
                let bit = dec.decode_bit(p)?;
                model.update(bit);
                byte = (byte << 1) | bit;
            }
            *b = byte;
        }
        out.write_all(&buf[..chunk])?;
        written += chunk;
    }
    out.flush()?;
    Ok(())
}
