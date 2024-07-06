use std::{path::PathBuf, process::Command};

use wanco::*;

const WRT_PATH: &str = "lib/cpp/wrt.o";
const LIB_PATH: &str = "lib/cpp/lib.o";

#[test]
fn test_fd_write() {
    let _ = env_logger::builder().try_init();

    let path = PathBuf::from("tests")
        .join("fd_write")
        .with_extension("wat");
    let tmp_filename = "wanco_fd_write";
    let obj = std::path::PathBuf::from("/tmp")
        .join(tmp_filename)
        .with_extension("o");
    let exe = std::path::PathBuf::from("/tmp").join(tmp_filename);

    // Compile
    let args = Args {
        input_file: path,
        // /tmp/<filename>.o
        output_file: obj.clone(),
        ..Default::default()
    };
    if let Err(e) = run_compiler(&args) {
        panic!("Could not compile {:?} ({})", &args.input_file, e);
    }
    // Link
    Command::new("g++")
        .arg(obj)
        .arg(WRT_PATH)
        .arg(LIB_PATH)
        .arg("-no-pie")
        .arg("-o")
        .arg(exe.clone())
        .output()
        .unwrap();

    // Execute
    let output = Command::new(exe).output().unwrap();

    // Assert
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Hello, World\n"));
}
