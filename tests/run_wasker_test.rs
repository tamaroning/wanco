use std::path::PathBuf;

use wanco::*;

const TEST_DIR: &'static str = "tests/wasker/wat/";

struct Context {
    pub success: Vec<String>,
    pub fail: Vec<String>,
}

#[test]
fn wasker_test() {
    env_logger::builder().init();

    let mut ctx = &mut Context {
        success: Vec::new(),
        fail: Vec::new(),
    };

    let entries = std::fs::read_dir(TEST_DIR).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        run_test(&mut ctx, &entry.path());
    }

    log::info!("### Summary ###");
    log::info!("Success: {} tests", ctx.success.len());
    log::info!("Fail: {} tests", ctx.fail.len());
    for f in &ctx.fail {
        log::error!("  - {:?}", f);
    }
    assert_eq!(ctx.fail.len(), 0);
}

fn run_test(ctx: &mut Context, path: &PathBuf) {
    let test_name = path.to_str().unwrap().to_string();

    let args = Args {
        input_file: std::path::PathBuf::from(path),
        output_file: std::path::PathBuf::from("/tmp/wasm.o"),
    };
    log::info!("Running test {:?}", &test_name);
    if let Err(e) = run_compiler(&args) {
        log::error!("Could not compile {:?} ({})", &args.input_file, e);
        ctx.fail.push(test_name);
    } else {
        ctx.success.push(test_name);
    }
}
