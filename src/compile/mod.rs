pub mod checkpoint;
mod compile_function;
mod compile_global;
mod compile_memory;
mod compile_module;
mod compile_type;
pub mod control;
pub mod helper;
mod synthesize;

use std::path;

use anyhow::{anyhow, Context as _, Result};
use inkwell::targets;

use compile_module::compile_module;

use crate::{context::Context, driver::Args};

pub fn compile(wasm: &[u8], args: &Args) -> Result<()> {
    // Create a new LLVM context and module
    let ictx = inkwell::context::Context::create();
    let module = ictx.create_module("wanco_aot");
    let builder = ictx.create_builder();
    let mut ctx = Context::new(args, &ictx, &module, builder);

    log::debug!("Start compilation");
    compile_module(wasm, &mut ctx)?;

    let obj_path = path::Path::new(&args.output_file.clone().unwrap_or("wasm.o".to_owned()))
        .with_extension("o");
    let asm_path = path::Path::new(&args.output_file.clone().unwrap_or("wasm.ll".to_owned()))
        .with_extension("ll");
    let tmp_obj_path = path::Path::new("/tmp/wasm.o");
    let exe_path = args.output_file.clone().unwrap_or("a.out".to_owned());
    let exe_path = path::Path::new(&exe_path);

    let target = get_host_target_machine().expect("Failed to get host architecture");

    if args.compile_only {
        log::debug!("write to {}", asm_path.display());
        ctx.module
            .print_to_file(asm_path.to_str().expect("error ll_path"))
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to write to the ll file")?;
        log::debug!("wrote to {}", asm_path.display());

        log::debug!("write to {}", obj_path.display());
        target
            .write_to_file(
                ctx.module,
                targets::FileType::Object,
                std::path::Path::new(obj_path.to_str().expect("error obj_path")),
            )
            .map_err(|e| anyhow!(e.to_string()))
            .context("Failed to write to the object file")?;
        log::debug!("wrote to {}", obj_path.display());

        return Ok(());
    }

    // Link
    let target = get_host_target_machine().expect("Failed to get host architecture");
    log::debug!("write to {}", tmp_obj_path.display());
    target
        .write_to_file(
            ctx.module,
            targets::FileType::Object,
            std::path::Path::new(tmp_obj_path.to_str().expect("error tmp_obj_path")),
        )
        .map_err(|e| anyhow!(e.to_string()))
        .context("Failed to write to the object file")?;
    log::debug!("wrote to {}", tmp_obj_path.display());

    log::debug!("linking object file");
    log::debug!(
        "c++ {} /usr/local/lib/libwanco.a -o {} -no-pie",
        tmp_obj_path.display(),
        exe_path.display(),
    );
    let o = std::process::Command::new("c++")
        .arg(tmp_obj_path)
        .arg("/usr/local/lib/libwanco.a")
        .arg("-o")
        .arg(exe_path)
        .arg("-no-pie")
        .output()
        .context("Failed to link object file")?;
    if !o.status.success() {
        let cc_stderr = String::from_utf8(o.stderr).unwrap();
        return Err(anyhow!("Failed to link object file: {}", cc_stderr));
    }

    Ok(())
}

fn get_host_target_machine() -> Result<targets::TargetMachine, String> {
    use targets::*;

    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| format!("failed to initialize native target: {}", e))?;

    let triple = TargetMachine::get_default_triple();
    let target =
        Target::from_triple(&triple).map_err(|e| format!("failed to get target: {}", e))?;

    let cpu = TargetMachine::get_host_cpu_name();
    let features = TargetMachine::get_host_cpu_features();

    let opt_level = inkwell::OptimizationLevel::None;
    let reloc_mode = RelocMode::Default;
    let code_model = CodeModel::Default;

    target
        .create_target_machine(
            &triple,
            cpu.to_str().expect("error get cpu info"),
            features.to_str().expect("error get features"),
            opt_level,
            reloc_mode,
            code_model,
        )
        .ok_or("failed to get target machine".to_string())
}
