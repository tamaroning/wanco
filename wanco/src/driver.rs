use anyhow::{anyhow, Context as _, Result};
use clap::Parser;
use inkwell::targets;
use std::path;

use crate::{
    compile::{self, cr_v2},
    context::Context,
};

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

    /// Enable full control-flow integrity
    #[arg(long, default_value = "false")]
    pub cf_protection: bool,

    /// Target (TODO: wip)
    #[arg(long)]
    pub target: Option<String>,

    /// Enable the checkpoint/restore feature. (v1)
    #[arg(long)]
    pub enable_cr: bool,

    /// Optimized migration points. (v1)
    #[arg(long)]
    pub optimize_cr: bool,

    /// Disable the loop checkpoint/restore feature. (v1)
    #[arg(long)]
    pub disable_loop_cr: bool,

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

    compile_and_link(&wasm, args)
}

pub fn check_config(args: &Args) -> bool {
    if (args.optimize_cr || args.disable_loop_cr) && !args.enable_cr {
        log::error!("Specify --enable-cr to enable checkpoint/restore feature (v1)");
        return false;
    }

    if args.enable_cr && (args.checkpoint_v2 || args.restore_v2) {
        log::error!("Cannot use both v1 and v2 checkpoint/restore features");
        return false;
    }

    true
}

pub fn compile_and_link(wasm: &[u8], args: &Args) -> Result<()> {
    // Create a new LLVM context and module
    let ictx = inkwell::context::Context::create();
    let module = ictx.create_module("wanco_aot");
    let builder = ictx.create_builder();
    let mut ctx = Context::new(args, &ictx, &module, builder);

    compile::compile_module(wasm, &mut ctx)?;

    if ctx.config.checkpoint_v2 || ctx.config.restore_v2 {
        log::info!("Writing module to memory buffer");
        let target = get_target_machine(args).map_err(|e| anyhow!(e))?;
        // TODO: remove
        {
            ctx.module
                .print_to_file("wasm.ll")
                .map_err(|e| anyhow!(e.to_string()))
                .context("Failed to write to the ll file")?;
            log::info!("wrote to wasm.ll");
        }
        // write module to memory
        let buf = target
            .write_to_memory_buffer(ctx.module, targets::FileType::Object)
            .map_err(|e| anyhow!(e.to_string()))?;
        log::info!("Wrote module to memory buffer");
        let obj = buf
            .create_object_file()
            .map_err(|()| anyhow!("Failed to create object file"))?;

        log::info!("Searching stackmap");
        let sections = obj.get_sections();
        let mut stackmap = None;
        for section in sections {
            let Some(name) = section.get_name() else {
                continue;
            };
            let name = name.to_str().expect("error get section name");
            if name != ".llvm_stackmaps" && name != "__llvm_stackmaps" {
                continue;
            }
            let data = section.get_contents();
            log::info!("Parsing stackmap");
            let res = cr_v2::stackmap::parse(data);
            stackmap = Some(res.map_err(|e| anyhow!(e))?);
        }

        if let Some(stkmap) = stackmap {
            cr_v2::stackmap::prettyprint(&stkmap);
        } else {
            log::error!("Failed to find .llvm_stackmap section");
        }
    }

    //let target = get_target_machine(args).map_err(|e| anyhow!(e))?;
    //let triple = target.get_triple();
    //log::debug!("target triple: {}", triple.as_str().to_str().unwrap());

    // TODO: Linker should use .bc file instead of .ll file.
    // Linker take .ll file for now since backward compatibility of the bc file is not guaranteed.
    // It enables to use the different version of clang as a linker.
    let llobj_path = path::Path::new(&args.output_file.clone().unwrap_or("wasm.o".to_owned()))
        .with_extension("bc");
    let asm_path = path::Path::new(&args.output_file.clone().unwrap_or("wasm.ll".to_owned()))
        .with_extension("ll");
    let random_suffix = rand::random::<u64>();
    let tmp_asm_path = format!("/tmp/wasm-{}.ll", random_suffix);
    let tmp_asm_path = path::Path::new(&tmp_asm_path);
    let tmp_llobj_path = format!("/tmp/wasm-{}.bc", random_suffix);
    let tmp_llobj_path = path::Path::new(&tmp_llobj_path);
    let exe_path = args.output_file.clone().unwrap_or("a.out".to_owned());
    let exe_path = path::Path::new(&exe_path);

    if args.compile_only {
        log::info!("Writing LLVM object");
        ctx.module
            .print_to_file(asm_path.to_str().expect("error ll_path"))
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to write to the ll file")?;
        log::info!("wrote to {}", asm_path.display());

        if !ctx.module.write_bitcode_to_path(&llobj_path) {
            return Err(anyhow!("Failed to write the LLVM object file"));
        }
        log::info!("wrote to {}", llobj_path.display());

        return Ok(());
    }

    // Write and Link object files
    log::info!("Writing LLVM object");
    if !ctx.module.write_bitcode_to_path(&tmp_llobj_path) {
        return Err(anyhow!("Failed to write to the LLVM object file"));
    }
    log::info!("wrote to {}", tmp_llobj_path.display());

    ctx.module
        .print_to_file(tmp_asm_path.to_str().expect("error ll_path"))
        .map_err(|e| anyhow!(e.to_string()))
        .context("Failed to write to the ll file")?;
    log::info!("wrote to {}", tmp_asm_path.display());

    log::info!("Linking object files");
    let clangxx = args.clang_path.clone().unwrap_or("clang++-17".to_owned());
    let library_path = args
        .library_path
        .clone()
        .unwrap_or("/usr/local/lib".to_owned());
    let mut cmd = std::process::Command::new(clangxx);
    let cmd = cmd
        .arg(tmp_asm_path)
        .arg(format!("{}/libwanco_rt.a", library_path))
        .arg(format!("{}/libwanco_wasi.a", library_path))
        .arg("-g")
        .arg("-o")
        .arg(exe_path)
        .arg("-no-pie")
        .arg(format!("-{}", args.optimization));

    // link protobuf
    cmd.arg("-lprotobuf");

    // link libunwind
    /*
    let triple = get_target_machine(args).unwrap().get_triple();
    let triple = triple.as_str().to_str().unwrap();
    if triple == "x86_64-unknown-linux-gnu" {
        cmd.arg("-lunwind");
        cmd.arg("-lunwind-x86_64");
    } else if triple == "aarch64-unknown-linux-gnu" {
        cmd.arg("-lunwind");
        cmd.arg("-lunwind-aarch64");
    }
    */

    if args.lto {
        cmd.arg("-flto");
    }
    if args.cf_protection {
        cmd.arg("-fcf-protection=full");
        //cmd.arg("-Wl,--enable-cet");
    }

    if let Some(ref target) = args.target {
        cmd.arg(format!("--target={}", target));
    }
    log::info!("{:?}", cmd);

    let o = cmd
        .output()
        .map_err(|e| anyhow!(e.to_string()))
        .context("Failed to link object files")?;
    if !o.status.success() {
        let cc_stderr = String::from_utf8(o.stderr).unwrap();
        return Err(anyhow!("Failed to link object files: {}", cc_stderr));
    }
    log::info!("Linked to {}", exe_path.display());

    Ok(())
}

fn get_target_machine(args: &Args) -> Result<targets::TargetMachine, String> {
    use targets::*;

    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| format!("failed to initialize native target: {}", e))?;

    let (cpu, target, triple, features) = if let Some(ref cpu) = args.target {
        let (triple, features) = match cpu.as_str() {
            "x86_64" => {
                Target::initialize_x86(&InitializationConfig::default());
                ("x86_64-linux-gnu", "+sse2")
            }
            // arm-linux-gnueabihf, aarch64-arm-linux-eabi, aarch64-linux-gnu
            "aarch64" => {
                Target::initialize_aarch64(&InitializationConfig::default());
                ("aarch64-arm-linux-eabi", "+neon,+fp-armv8,+simd")
            }

            _ => ("x86_64-unknown-linux-gnu", "+sse2"),
        };
        let triple = TargetTriple::create(triple);
        let target =
            Target::from_triple(&triple).map_err(|e| format!("failed to get target: {}", e))?;
        (cpu.to_owned(), target, triple, features.to_owned())
    } else {
        let triple = TargetMachine::get_default_triple();
        let target =
            Target::from_triple(&triple).map_err(|e| format!("failed to get target: {}", e))?;
        let cpu = TargetMachine::get_host_cpu_name()
            .to_str()
            .expect("error get cpu info")
            .to_owned();
        let features = TargetMachine::get_host_cpu_features().to_owned();
        let features = features.to_str().expect("error get features").to_owned();
        (cpu, target, triple, features)
    };

    let opt_level = match &args.optimization {
        crate::driver::OptimizationLevel::O0 => inkwell::OptimizationLevel::None,
        crate::driver::OptimizationLevel::O1 => inkwell::OptimizationLevel::Less,
        crate::driver::OptimizationLevel::O2 => inkwell::OptimizationLevel::Default,
        crate::driver::OptimizationLevel::O3 => inkwell::OptimizationLevel::Aggressive,
    };
    let reloc_mode = RelocMode::Default;
    let code_model = CodeModel::Default;

    target
        .create_target_machine(&triple, &cpu, &features, opt_level, reloc_mode, code_model)
        .ok_or("failed to get target machine".to_string())
}
