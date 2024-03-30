mod common;
use common::*;
use regex::bytes::Regex;

#[test]
fn prints_help() {
    let output = rsid3_run(&["--help"]);
    assert!(output.status.success());
    assert!(output.stdout.starts_with(b"Usage:"));

    let output = rsid3_run(&["-h"]);
    assert!(output.status.success());
    assert!(output.stdout.starts_with(b"Usage:"));
}

#[test]
fn prints_version() {
    let expected_prefix = &[
        b"rsid3 ",
        env!("RSID3_VERSION_STR").as_bytes(),
        b"-",
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
fn prints_supported_frames() {
    let re = Regex::new(
        r"(?s)Read-write frames.*COMM.*TIT2.*Read-only frames.*APIC.*USER"
    ).unwrap();

    let output = rsid3_run(&["--list-frames"]);
    assert!(re.is_match(&output.stdout));

    let output = rsid3_run(&["-L"]);
    assert!(re.is_match(&output.stdout));
}

#[test]
fn gets_empty() {
    let file = TestFile::empty();
    let output = rsid3_run(&[file.path()]);
    assert_eq!(output.stderr, [
        file.path().as_os_str().as_encoded_bytes(),
        b": No tag found\n",
    ].concat());
}
