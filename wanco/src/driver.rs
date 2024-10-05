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

impl std::fmt::Display for OptimizationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptimizationLevel::O0 => write!(f, "O0"),
            OptimizationLevel::O1 => write!(f, "O1"),
            OptimizationLevel::O2 => write!(f, "O2"),
            OptimizationLevel::O3 => write!(f, "O3"),
        }
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

    /// Enable LTO.
    #[arg(long, default_value = "false")]
    pub lto: bool,

    /// Target (TODO: wip)
    #[arg(long)]
    pub target: Option<String>,

    /// Enable the checkpoint/restore feature. (v1)
    #[arg(long)]
    pub enable_cr: bool,

    /// Enable C/R for loop. (v1)
    #[arg(long)]
    pub enable_loop_cr: bool,

    /// TODO: Enable the checkpoint feature. (v2)
    #[arg(long)]
    pub checkpoint_v2: bool,

    /// TODO: Enable the restore feature. (v2)
    #[arg(long)]
    pub restore_v2: bool,

    /// Optimization level.
    #[arg(short = 'O', value_enum, default_value = "1")]
    pub optimization: OptimizationLevel,

    /// Custom path to clang or clang++. (default to clang++)
    #[arg(long)]
    pub clang_path: Option<String>,

    /// Library path. (default to /usr/local/lib on Unix)
    #[arg(short)]
    pub library_path: Option<String>,
}

pub fn run_compiler(args: &Args) -> Result<()> {
    let buf: Vec<u8> = std::fs::read(&args.input_file)
        .with_context(|| format!("Failed to open {:?}", args.input_file))?;
    // Parse the input file into a wasm module binary
    let wasm = wat::parse_bytes(&buf)?;
    assert!(wasm.starts_with(b"\0asm"));

    compile::compile(&wasm, args)
}

pub fn check_config(args: &Args) -> bool {
    if (args.enable_cr || args.enable_loop_cr) && (args.checkpoint_v2 || args.restore_v2) {
        log::error!("Cannot use both v1 and v2 checkpoint/restore features");
        return false;
    }

    /*
    if args.enable_cr && !args.enable_cr {
        log::error!("specify both --enable-cr and --enable-loop-cr");
        return false;
    }
    */

    true
}
