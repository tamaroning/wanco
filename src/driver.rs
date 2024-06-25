use std::path;

use clap::Parser;

use anyhow::{Context as _, Result};

use crate::compile;

#[derive(Parser, Debug)]
pub struct Args {
    pub input_file: path::PathBuf,

    #[arg(short, long, default_value = "./wasm.o")]
    pub output_file: path::PathBuf,
}

pub fn run(args: &Args) -> Result<()> {
    let buf: Vec<u8> = std::fs::read(&args.input_file)
        .with_context(|| format!("Failed to open {:?}", args.input_file))?;
    // Parse the input file into a wasm module binary
    let wasm = wat::parse_bytes(&buf)?;
    assert!(wasm.starts_with(b"\0asm"));

    compile::compile(&wasm, args)
}
