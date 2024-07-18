use std::path;

use clap::Parser;

use anyhow::{Context as _, Result};

use crate::compile;

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum OptimizationLevel {
    #[clap(name = "0")]
    O0,
    #[clap(name = "1")]
    O1,
    #[clap(name = "2")]
    O2,
    #[clap(name = "3")]
    O3,
}

impl Default for OptimizationLevel {
    fn default() -> Self {
        OptimizationLevel::O1
    }
}

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

    #[arg(short = 'O', value_enum, default_value = "2")]
    pub optimization: OptimizationLevel,
}

pub fn run_compiler(args: &Args) -> Result<()> {
    let buf: Vec<u8> = std::fs::read(&args.input_file)
        .with_context(|| format!("Failed to open {:?}", args.input_file))?;
    // Parse the input file into a wasm module binary
    let wasm = wat::parse_bytes(&buf)?;
    assert!(wasm.starts_with(b"\0asm"));

    compile::compile(&wasm, args)
}
