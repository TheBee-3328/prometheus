//! Compressor: order-2 + word model + 32-bit arithmetic coder.

use prometheus_core::{ArithmeticEncoder, CombinedModel, Model, MAGIC};
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

    if let Err(e) = compress(&input, &output) {
        eprintln!("prometheus-compress: {e}");
        process::exit(1);
    }
}

fn usage() -> ! {
    eprintln!("usage: prometheus-compress <input> <output>");
    process::exit(2);
}

fn compress(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut input = BufReader::new(File::open(input_path)?);
    let mut data = Vec::new();
    input.read_to_end(&mut data)?;

    let out = BufWriter::new(File::create(output_path)?);
    let mut out = out;
    out.write_all(MAGIC)?;
    out.write_all(&(data.len() as u64).to_le_bytes())?;

    let mut model = CombinedModel::new();
    let mut enc = ArithmeticEncoder::new(out);
    for &byte in &data {
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1;
            let p = model.predict();
            enc.encode_bit(bit, p)?;
            model.update(bit);
        }
    }
    let mut out = enc.finish()?;
    out.flush()?;
    Ok(())
}
