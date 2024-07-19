use std::{path::PathBuf, process::Command};

use wanco::*;

const LIBS: [&str; 2] = [
    "/usr/local/lib/libwanco_rt.a",
    "/usr/local/lib/libwanco_wasi.a",
];

#[test]
fn test_fd_write() {
    let _ = env_logger::builder().try_init();

    let path = PathBuf::from("tests")
        .join("fd_write")
        .with_extension("wat");
    let tmp_filename = "wanco_fd_write";
    let obj = std::path::PathBuf::from("/tmp")
        .join(tmp_filename)
        .with_extension("o")
        .to_str()
        .unwrap()
        .to_string();
    let exe = std::path::PathBuf::from("/tmp").join(tmp_filename);

    // Compile
    let args = Args {
        input_file: path,
        // /tmp/<filename>.o
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
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Hello, World\n"));
}
