mod common;
use common::*;

#[test]
fn prints_help() {
    let output = rsid3_run(&["--help"]);
    assert!(output.status.success());
    assert!(output.stdout.starts_with("Usage:".as_bytes()));

    let output = rsid3_run(&["-h"]);
    assert!(output.status.success());
    assert!(output.stdout.starts_with("Usage:".as_bytes()));
}

#[test]
fn prints_version() {
    let expected_prefix = &[
        "rsid3 ".as_bytes(),
        env!("RSID3_VERSION_STR").as_bytes(),
        "-".as_bytes(),
        env!("VERGEN_GIT_SHA").chars().take(8).collect::<String>().as_bytes(),
    ].concat();

    let output = rsid3_run(&["--version"]);
    assert!(output.status.success());
    assert!(output.stdout.starts_with(expected_prefix));

    let output = rsid3_run(&["-V"]);
    assert!(output.status.success());
    assert!(output.stdout.starts_with(expected_prefix));
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
