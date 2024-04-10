mod common;
use common::*;
use regex::bytes::Regex;
use std::ffi::OsStr;

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

#[test]
fn prints_all_frames() {
    let file = TestFile::tit2();
    let output = rsid3_run(&[file.path()]);
    assert_eq!(output.stdout, [
        file.path().as_os_str().as_encoded_bytes(),
        b": ID3v2.4, 1 frame:\n",
        b"TIT2: Sample Title\n",
    ].concat());

    let file = TestFile::txxx();
    let output = rsid3_run(&[file.path()]);
    assert_eq!(output.stdout, [
        file.path().as_os_str().as_encoded_bytes(),
        b": ID3v2.4, 1 frame:\n",
        b"TXXX[Description]: Sample Content\n",
    ].concat());

    let file = TestFile::comm();
    let output = rsid3_run(&[file.path()]);
    assert_eq!(output.stdout, [
        file.path().as_os_str().as_encoded_bytes(),
        b": ID3v2.4, 1 frame:\n",
        b"COMM[Description](eng): Sample Content\n",
    ].concat());

    let file = TestFile::nirvana();
    let re_artist = Regex::new(r"(?m)^TPE1: Nirvana$").unwrap();
    let re_album_artist = Regex::new(r"(?m)^TPE2: Nirvana$").unwrap();
    let re_album = Regex::new(r"(?m)^TALB: Nevermind$").unwrap();
    let re_title = Regex::new(r"(?m)^TIT2: Smells Like Teen Spirit$").unwrap();
    let re_date = Regex::new(r"(?m)^TDOR: 1991$").unwrap();
    let re_track = Regex::new(r"(?m)^TRCK: 01/13$").unwrap();
    let re_genre = Regex::new(r"(?m)^TCON: Grunge Rock$").unwrap();
    let output = rsid3_run(&[file.path()]);
    assert!(re_artist.is_match(&output.stdout));
    assert!(re_album_artist.is_match(&output.stdout));
    assert!(re_album.is_match(&output.stdout));
    assert!(re_title.is_match(&output.stdout));
    assert!(re_date.is_match(&output.stdout));
    assert!(re_track.is_match(&output.stdout));
    assert!(re_genre.is_match(&output.stdout));
    assert!(output.stdout.starts_with(&[
        file.path().as_os_str().as_encoded_bytes(),
        b": ID3v2.4, 7 frames:\n",
    ].concat()));
    assert_eq!(output.stdout.iter().filter(|&&x| x == b'\n').count(), 8);
}

#[test]
fn prints_single_frame() {
    let file = TestFile::tit2();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TIT2"), fpath]);
    assert_eq!(output.stdout, b"Sample Title");

    let file = TestFile::txxx();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TXXX"), OsStr::new("Description"), fpath]);
    assert_eq!(output.stdout, b"Sample Content");

    let file = TestFile::comm();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--COMM"), OsStr::new("Description"), OsStr::new("eng"), fpath]);
    assert_eq!(output.stdout, b"Sample Content");

    let file = TestFile::nirvana();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TIT2"), fpath]);
    assert_eq!(output.stdout, b"Smells Like Teen Spirit");
    let output = rsid3_run(&[OsStr::new("--TPE1"), fpath]);
    assert_eq!(output.stdout, b"Nirvana");
    let output = rsid3_run(&[OsStr::new("--TRCK"), fpath]);
    assert_eq!(output.stdout, b"01/13");
}
