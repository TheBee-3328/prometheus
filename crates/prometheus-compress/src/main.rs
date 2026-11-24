//! Compressor: preprocessor + order-2 + LZ match model + 32-bit arithmetic coder.

use prometheus_core::{encode, ArithmeticEncoder, Model, OrdersMatchModel, MAGIC};
use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::process;

fn main() {
    let mut args = env::args_os().skip(1);
    let Some(input) = args.next() else {
        process::exit(2);
    };
    let Some(output) = args.next() else {
        process::exit(2);
    };
    if args.next().is_some() {
        process::exit(2);
    }

    if compress(&input, &output).is_err() {
        process::exit(1);
    }
}

fn compress(input_path: &std::ffi::OsStr, output_path: &std::ffi::OsStr) -> io::Result<()> {
    let mut input = BufReader::new(File::open(input_path)?);
    let mut raw = Vec::new();
    input.read_to_end(&mut raw)?;
    let data = encode(&raw);

    let out = BufWriter::new(File::create(output_path)?);
    let mut out = out;
    out.write_all(MAGIC)?;
    out.write_all(&(data.len() as u64).to_le_bytes())?;

    let mut model = OrdersMatchModel::new();
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
