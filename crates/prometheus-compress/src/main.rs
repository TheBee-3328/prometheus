//! Phase 0 store baseline: write magic + length + raw payload.
//! Format is intentional so Phase 1 can replace the payload encoding
//! without changing the CLI contract.

use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process;

const MAGIC: &[u8; 4] = b"PRM0";

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
    let mut input = File::open(input_path)?;
    let mut data = Vec::new();
    input.read_to_end(&mut data)?;

    let mut out = File::create(output_path)?;
    out.write_all(MAGIC)?;
    out.write_all(&(data.len() as u64).to_le_bytes())?;
    out.write_all(&data)?;
    out.flush()?;
    Ok(())
}
