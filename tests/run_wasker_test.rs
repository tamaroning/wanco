use std::path::PathBuf;

use wanco::*;

const TEST_DIR: &'static str = "tests/wasker/wat/";

macro_rules! ident_to_str {
    ($ident:ident) => {
        match stringify!($ident) {
            s if s.starts_with("r#") => s.trim_start_matches("r#"),
            other => other,
        }
    };
}

macro_rules! wasker_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            wasker_test(ident_to_str!($name));
        }
    };
}

fn wasker_test(test_name: &str) {
    let wat_path = PathBuf::from(TEST_DIR)
        .join(test_name)
        .with_extension("wat");
    run_test(&wat_path);
}

fn run_test(path: &PathBuf) {
    let _ = env_logger::builder().try_init();
    let test_name = path.to_str().unwrap().to_string();

    let args = Args {
        input_file: std::path::PathBuf::from(path),
        output_file: std::path::PathBuf::from("/tmp/wasm.o"),
    };
    log::info!("Running test {:?}", &test_name);
    if let Err(e) = run_compiler(&args) {
        log::error!("Could not compile {:?} ({})", &args.input_file, e);
        panic!();
    }
}

wasker_test!(address32);
wasker_test!(address64);
wasker_test!(align);
wasker_test!(block);
wasker_test!(br);
wasker_test!(br_if);
wasker_test!(br_table);
wasker_test!(bulk);
wasker_test!(call);
wasker_test!(call_indirect);
wasker_test!(convert);
wasker_test!(endianness);
wasker_test!(example);
wasker_test!(r#f64);
wasker_test!(f64_bitwise);
wasker_test!(f64_cmp);
wasker_test!(r#i64);
wasker_test!(local_get);
wasker_test!(memory_copy);
wasker_test!(memory_fill);
wasker_test!(memory_size);
wasker_test!(r#if);
wasker_test!(r#loop);
wasker_test!(r#return);
wasker_test!(select);
wasker_test!(switch);
