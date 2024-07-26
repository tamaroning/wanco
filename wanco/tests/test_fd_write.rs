use std::{path::PathBuf, process::Command};

use wanco::*;

#[test]
fn test_fd_write() {
    let _ = env_logger::builder().try_init();

    let path = PathBuf::from("tests")
        .join("fd_write")
        .with_extension("wat");
    let exe = std::path::PathBuf::from("/tmp").join("wanco_fd_write");

    // Compile
    let args = Args {
        input_file: path,
        // /tmp/<filename>.o
        output_file: Some(exe.to_str().unwrap().to_owned()),
        ..Default::default()
    };
    if let Err(e) = run_compiler(&args) {
        panic!("Could not compile {:?} ({})", &args.input_file, e);
    }
    // Execute
    let output = Command::new(exe).output().unwrap();

    // Assert
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stdout.contains("Hello, World\n"));
    assert!(!stderr.contains("Error"));
}
