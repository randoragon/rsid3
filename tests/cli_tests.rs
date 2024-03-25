mod common;
use common::*;

#[test]
fn prints_help() {
    let output = rsid3_run(&["--help"]);
    assert!(output.status.success());
    assert!(output.stdout.starts_with("Usage:".as_bytes()));
}

#[test]
fn gets_text() {
    let file = TestFile::empty();
    let output = rsid3_run(&["--"]);
}
