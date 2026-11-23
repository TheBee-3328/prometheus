//! Phase 0 store baseline decompressor.
//! Reads only the compressed archive (magic + length + payload).
//! Does not open or read the original input file.

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
    let mut input = File::open(input_path)?;
    let mut buf = Vec::new();
    input.read_to_end(&mut buf)?;

    if buf.len() < 12 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "archive too short",
        ));
    }
    if &buf[0..4] != MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "bad magic",
        ));
    }

    let mut len_bytes = [0u8; 8];
    len_bytes.copy_from_slice(&buf[4..12]);
    let declared = u64::from_le_bytes(len_bytes) as usize;
    let payload = &buf[12..];
    if payload.len() != declared {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "length mismatch: header={declared} payload={}",
                payload.len()
            ),
        ));
    }

    let mut out = File::create(output_path)?;
    out.write_all(payload)?;
    out.flush()?;
    Ok(())
}
