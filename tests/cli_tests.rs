mod common;
use common::*;

#[test]
fn prints_help() {
    let output = rsid3_run(&["--help"]);
    assert!(output.status.success());
    assert!(output.stdout.starts_with("Usage:".as_bytes()));
}

#[test]
fn gets_empty() {
    let file = TestFile::empty();
    let output = rsid3_run(&[file.path()]);
    assert_eq!(output.stderr, [
        file.path().as_os_str().as_encoded_bytes(),
        ": No tag found\n".as_bytes(),
    ].concat());
}
