use std::{path::PathBuf, process::Command};

use wanco::*;

const TEST_DIR: &str = "tests/wasker/";
const LIBS: [&str; 2] = [
    "/usr/local/lib/libwanco_rt.a",
    "/usr/local/lib/libwanco_wasi.a",
];

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
            run_test(ident_to_str!($name));
        }
    };
}

fn run_test(test_name: &str) {
    let _ = env_logger::builder().try_init();

    let path = PathBuf::from(TEST_DIR)
        .join(test_name)
        .with_extension("wat");
    let tmp_filename = format!("wanco_wasker_{}", test_name);
    let obj = std::path::PathBuf::from("/tmp")
        .join(tmp_filename.clone())
        .with_extension("o")
        .to_str()
        .unwrap()
        .to_string();
    let exe = std::path::PathBuf::from("/tmp").join(tmp_filename);

    // Compile
    let args = Args {
        input_file: path,
        output_file: Some(obj.clone()),
        compile_only: true,
        ..Default::default()
    };
    if let Err(e) = run_compiler(&args) {
        panic!("Could not compile {:?} ({})", &args.input_file, e);
    }
    // Link
    let mut cmd = Command::new("g++");
    let mut cmd = cmd.arg(obj).arg("-no-pie").arg("-o").arg(exe.clone());
    for lib in LIBS.iter() {
        cmd = cmd.arg(lib);
    }
    cmd.output().unwrap();

    // Execute
    let output = Command::new(exe).output().unwrap();

    // Assert
    // "#Test Failed" means "should fail"
    let stdout = String::from_utf8(output.stdout)
        .unwrap()
        .replace("#Test Failed", "");
    assert!(!stdout.contains("Test Failed"));
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