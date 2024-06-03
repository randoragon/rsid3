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
    assert!(output.status.success());
    assert!(re.is_match(&output.stdout));

    let output = rsid3_run(&["-L"]);
    assert!(output.status.success());
    assert!(re.is_match(&output.stdout));
}

#[test]
fn prints_no_tag() {
    let file = TestFile::empty();
    let output = rsid3_run(&[file.path()]);
    assert!(output.status.success());
    assert_eq!(output.stderr, [
        file.path().as_os_str().as_encoded_bytes(),
        b": No tag found\n",
    ].concat());
}

#[test]
fn prints_all_frames() {
    let file = TestFile::tit2();
    let output = rsid3_run(&[file.path()]);
    assert!(output.status.success());
    assert_eq!(output.stdout, [
        file.path().as_os_str().as_encoded_bytes(),
        b": ID3v2.4, 1 frame:\n",
        b"TIT2: Sample Title\n",
    ].concat());

    let file = TestFile::txxx();
    let output = rsid3_run(&[file.path()]);
    assert!(output.status.success());
    assert_eq!(output.stdout, [
        file.path().as_os_str().as_encoded_bytes(),
        b": ID3v2.4, 1 frame:\n",
        b"TXXX[Description]: Sample Content\n",
    ].concat());

    let file = TestFile::comm();
    let output = rsid3_run(&[file.path()]);
    assert!(output.status.success());
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
    assert!(output.status.success());
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
fn prints_all_frames_multiple_files() {
    let file1 = TestFile::tit2();
    let file2 = TestFile::txxx();
    let fpath1 = file1.path().as_os_str();
    let fpath2 = file2.path().as_os_str();

    let output = rsid3_run(&[fpath1, fpath2]);
    assert!(output.status.success());
    assert_eq!(output.stdout, [
        fpath1.as_encoded_bytes(), b": ID3v2.4, 1 frame:\n",
        b"TIT2: Sample Title\n\n",
        fpath2.as_encoded_bytes(), b": ID3v2.4, 1 frame:\n",
        b"TXXX[Description]: Sample Content\n",
    ].concat());

    let output = rsid3_run(&[OsStr::new("--"), fpath2, fpath1]);
    assert!(output.status.success());
    assert_eq!(output.stdout, [
        fpath2.as_encoded_bytes(), b": ID3v2.4, 1 frame:\n",
        b"TXXX[Description]: Sample Content\n\n",
        fpath1.as_encoded_bytes(), b": ID3v2.4, 1 frame:\n",
        b"TIT2: Sample Title\n",
    ].concat());
}

#[test]
fn prints_single_frame() {
    let file = TestFile::tit2();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TIT2"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Sample Title");

    let file = TestFile::txxx();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TXXX"), OsStr::new("Description"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Sample Content");

    let file = TestFile::comm();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--COMM"), OsStr::new("Description"), OsStr::new("eng"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Sample Content");

    let file = TestFile::nirvana();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TIT2"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Smells Like Teen Spirit");
    let output = rsid3_run(&[OsStr::new("--TPE1"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Nirvana");
    let output = rsid3_run(&[OsStr::new("--TRCK"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"01/13");
}

#[test]
fn prints_missing_frame() {
    let file = TestFile::tit2();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TPE1"), fpath]);
    assert!(output.stdout.is_empty());
    let output = rsid3_run(&[OsStr::new("--TXXX"), OsStr::new("abc"), fpath]);
    assert!(output.stdout.is_empty());
    let output = rsid3_run(&[OsStr::new("--COMM"), OsStr::new("abc"), OsStr::new("eng"), fpath]);
    assert!(output.stdout.is_empty());
}

#[test]
fn prints_multiple_frames() {
    let file = TestFile::tit2();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TIT2"), OsStr::new("--TPE1"), OsStr::new("--TALB"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Sample Title\n\n");

    let file = TestFile::nirvana();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TIT2"), OsStr::new("--TPE1"), OsStr::new("--TALB"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Smells Like Teen Spirit\nNirvana\nNevermind");
}

#[test]
fn prints_multiple_frames_with_delimiter() {
    let file = TestFile::tit2();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("-d,"), OsStr::new("--TIT2"), OsStr::new("--TPE1"), OsStr::new("--TALB"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Sample Title,,");
    let output = rsid3_run(&[OsStr::new("-0d"), OsStr::new("--TIT2"), OsStr::new("--TPE1"), OsStr::new("--TALB"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Sample Title\0\0");

    let file = TestFile::nirvana();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TIT2"), OsStr::new("--TPE1"), OsStr::new("--TALB"), OsStr::new("-d"), OsStr::new("abc"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Smells Like Teen SpiritabcNirvanaabcNevermind");
}

#[test]
fn prints_single_frame_multiple_files() {
    let file1 = TestFile::tit2();
    let file2 = TestFile::nirvana();
    let fpath1 = file1.path().as_os_str();
    let fpath2 = file2.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TIT2"), fpath1, fpath2]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Sample Title\nSmells Like Teen Spirit");

    let output = rsid3_run(&[OsStr::new("--TIT2"), fpath2, fpath1]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Smells Like Teen Spirit\nSample Title");
}

#[test]
fn prints_single_frame_multiple_files_with_delimiter() {
    let file1 = TestFile::tit2();
    let file2 = TestFile::nirvana();
    let fpath1 = file1.path().as_os_str();
    let fpath2 = file2.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TIT2"), OsStr::new("-D,"), fpath1, fpath2]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Sample Title,Smells Like Teen Spirit");
    let output = rsid3_run(&[OsStr::new("--TIT2"), OsStr::new("-0D"), fpath1, fpath2]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Sample Title\0Smells Like Teen Spirit");

    let output = rsid3_run(&[OsStr::new("-D"), OsStr::new("abc"), OsStr::new("--TIT2"), fpath2, fpath1]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Smells Like Teen SpiritabcSample Title");
}

#[test]
fn prints_multiple_frames_multiple_files_with_delimiters() {
    let file1 = TestFile::tit2();
    let file2 = TestFile::nirvana();
    let fpath1 = file1.path().as_os_str();
    let fpath2 = file2.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("-d,"), OsStr::new("-Dabc"), OsStr::new("--TIT2"), OsStr::new("--TPE1"), OsStr::new("--TALB"), fpath1, fpath2]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Sample Title,,abcSmells Like Teen Spirit,Nirvana,Nevermind");

    let output = rsid3_run(&[OsStr::new("-0d"), OsStr::new("-0D"), OsStr::new("--TIT2"), OsStr::new("--TPE1"), OsStr::new("--TALB"), fpath1, fpath2]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Sample Title\0\0\0Smells Like Teen Spirit\0Nirvana\0Nevermind");
}

#[test]
fn sets_text_frame() {
    let file = TestFile::empty();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TIT2="), OsStr::new("new title"), fpath]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());

    let output = rsid3_run(&[OsStr::new("--TIT2"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"new title");
}

#[test]
fn sets_txxx_frame() {
    let file = TestFile::empty();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TXXX="), OsStr::new("desc"), OsStr::new("content"), fpath]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());

    let output = rsid3_run(&[OsStr::new("--TXXX"), OsStr::new("desc"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"content");
}

#[test]
fn sets_comm_frame() {
    let file = TestFile::empty();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--COMM="), OsStr::new("desc"), OsStr::new("eng"), OsStr::new("content"), fpath]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());

    let output = rsid3_run(&[OsStr::new("--COMM"), OsStr::new("desc"), OsStr::new("eng"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"content");
}

#[test]
fn deletes_text_frame() {
    let file = TestFile::tit2();
    let fpath = file.path().as_os_str();

    let output = rsid3_run(&[OsStr::new("--TIT2"), fpath]);
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());

    let output = rsid3_run(&[OsStr::new("--TIT2-"), fpath]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());

    let output = rsid3_run(&[OsStr::new("--TIT2"), fpath]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn deletes_txxx_frame() {
    let file = TestFile::txxx();
    let fpath = file.path().as_os_str();

    let output = rsid3_run(&[OsStr::new("--TXXX"), OsStr::new("Description"), fpath]);
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());

    let output = rsid3_run(&[OsStr::new("--TXXX-"), OsStr::new("Description"), fpath]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());

    let output = rsid3_run(&[OsStr::new("--TXXX"), OsStr::new("Description"), fpath]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn deletes_comm_frame() {
    let file = TestFile::comm();
    let fpath = file.path().as_os_str();

    let output = rsid3_run(&[OsStr::new("--COMM"), OsStr::new("Description"), OsStr::new("eng"), fpath]);
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());

    let output = rsid3_run(&[OsStr::new("--COMM-"), OsStr::new("Description"), OsStr::new("eng"), fpath]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());

    let output = rsid3_run(&[OsStr::new("--COMM"), OsStr::new("Description"), OsStr::new("eng"), fpath]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn executes_actions_in_passed_order() {
    // Print title and then change it
    let file = TestFile::nirvana();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TIT2"), OsStr::new("--TIT2="), OsStr::new("new title"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Smells Like Teen Spirit");

    // Change title and then print it
    let file = TestFile::nirvana();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TIT2="), OsStr::new("new title"), OsStr::new("--TIT2"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"new title");
}
