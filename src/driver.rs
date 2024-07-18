use std::path;

use clap::Parser;

use anyhow::{Context as _, Result};

use crate::compile;

#[derive(Parser, Debug, Default)]
pub struct Args {
    pub input_file: path::PathBuf,

    /// Place the output file.
    #[arg(short, long)]
    pub output_file: Option<String>,

    /// Compile and assemble, but do not link.
    #[arg(short)]
    pub compile_only: bool,

    /// Target (TODO: wip)
    #[arg(long)]
    pub target: Option<String>,

    /// Enable the checkpoint feature.
    #[arg(long)]
    pub checkpoint: bool,

    /// Enable the restore feature.
    #[arg(long)]
    pub restore: bool,
}

pub fn run_compiler(args: &Args) -> Result<()> {
    let buf: Vec<u8> = std::fs::read(&args.input_file)
        .with_context(|| format!("Failed to open {:?}", args.input_file))?;
    // Parse the input file into a wasm module binary
    let wasm = wat::parse_bytes(&buf)?;
    assert!(wasm.starts_with(b"\0asm"));

    compile::compile(&wasm, args)
}
