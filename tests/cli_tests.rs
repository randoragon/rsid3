// rsid3 - a simple, command line ID3v2 tag editor designed for scripting
// Copyright (C) 2024  Randoragon
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
mod common;
use common::*;
use regex::bytes::Regex;
use std::ffi::OsStr;
use rsid3::ExitCode;

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
        env!("CARGO_PKG_VERSION").as_bytes(),
        b"+",
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
        r"(?s)ID3 standard.*2\.2, 2\.3, 2\.4.*COMM.*TIT2.*r/w.*http://id3.org/"
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

    let output = rsid3_run(&[fpath2, fpath1]);
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
    assert!(output.status.code().unwrap() == ExitCode::FrameNotFound as i32);
    assert!(output.stdout.is_empty());
    let output = rsid3_run(&[OsStr::new("--TXXX"), OsStr::new("abc"), fpath]);
    assert!(output.status.code().unwrap() == ExitCode::FrameNotFound as i32);
    assert!(output.stdout.is_empty());
    let output = rsid3_run(&[OsStr::new("--COMM"), OsStr::new("abc"), OsStr::new("eng"), fpath]);
    assert!(output.status.code().unwrap() == ExitCode::FrameNotFound as i32);
    assert!(output.stdout.is_empty());
}

#[test]
fn prints_multiple_frames() {
    let file = TestFile::tit2();
    let fpath = file.path().as_os_str();
    let output = rsid3_run(&[OsStr::new("--TIT2"), OsStr::new("--TPE1"), OsStr::new("--TALB"), fpath]);
    assert!(output.status.code().unwrap() == ExitCode::FrameNotFound as i32);
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
    assert!(output.status.code().unwrap() == ExitCode::FrameNotFound as i32);
    assert_eq!(output.stdout, b"Sample Title,,");
    let output = rsid3_run(&[OsStr::new("-0d"), OsStr::new("--TIT2"), OsStr::new("--TPE1"), OsStr::new("--TALB"), fpath]);
    assert!(output.status.code().unwrap() == ExitCode::FrameNotFound as i32);
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
    assert!(output.status.code().unwrap() == ExitCode::FrameNotFound as i32);
    assert_eq!(output.stdout, b"Sample Title,,abcSmells Like Teen Spirit,Nirvana,Nevermind");

    let output = rsid3_run(&[OsStr::new("-0d"), OsStr::new("-0D"), OsStr::new("--TIT2"), OsStr::new("--TPE1"), OsStr::new("--TALB"), fpath1, fpath2]);
    assert!(output.status.code().unwrap() == ExitCode::FrameNotFound as i32);
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
    assert!(output.status.code().unwrap() == ExitCode::FrameNotFound as i32);
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
    assert!(output.status.code().unwrap() == ExitCode::FrameNotFound as i32);
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
    assert!(output.status.code().unwrap() == ExitCode::FrameNotFound as i32);
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

#[test]
fn versioned_samples_contain_correctly_versioned_tags() {
    let file2_2 = TestFile::id3v2_2();
    let file2_3 = TestFile::id3v2_3();
    let file2_4 = TestFile::id3v2_4();
    let fpath2_2 = file2_2.path().as_os_str();
    let fpath2_3 = file2_3.path().as_os_str();
    let fpath2_4 = file2_4.path().as_os_str();

    // Make sure the files have correct tag version
    let output = rsid3_run(&[fpath2_2]);
    assert!(output.status.success());
    assert!(output.stdout.starts_with(&[
        file2_2.path().as_os_str().as_encoded_bytes(),
        b": ID3v2.2, 1 frame:\n",
    ].concat()));

    let output = rsid3_run(&[fpath2_3]);
    assert!(output.status.success());
    assert!(output.stdout.starts_with(&[
        file2_3.path().as_os_str().as_encoded_bytes(),
        b": ID3v2.3, 2 frames:\n",
    ].concat()));

    let output = rsid3_run(&[fpath2_4]);
    assert!(output.status.success());
    assert!(output.stdout.starts_with(&[
        file2_4.path().as_os_str().as_encoded_bytes(),
        b": ID3v2.4, 2 frames:\n",
    ].concat()));
}

#[test]
fn converts_between_tag_versions_noop() {
    let file2_2 = TestFile::id3v2_2();
    let file2_3 = TestFile::id3v2_3();
    let file2_4 = TestFile::id3v2_4();
    let fpath2_2 = file2_2.path().as_os_str();
    let fpath2_3 = file2_3.path().as_os_str();
    let fpath2_4 = file2_4.path().as_os_str();

    // Attempting to convert to the same version should not change the file contents whatsoever.
    let file2_2_old_content = std::fs::read(fpath2_2).unwrap();
    let output = rsid3_run(&[OsStr::new("--id3v2.2"), fpath2_2]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let file2_2_new_content = std::fs::read(fpath2_2).unwrap();
    assert_eq!(file2_2_old_content, file2_2_new_content);

    let file2_3_old_content = std::fs::read(fpath2_3).unwrap();
    let output = rsid3_run(&[OsStr::new("--id3v2.3"), fpath2_3]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let file2_3_new_content = std::fs::read(fpath2_3).unwrap();
    assert_eq!(file2_3_old_content, file2_3_new_content);

    let file2_4_old_content = std::fs::read(fpath2_4).unwrap();
    let output = rsid3_run(&[OsStr::new("--id3v2.4"), fpath2_4]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let file2_4_new_content = std::fs::read(fpath2_4).unwrap();
    assert_eq!(file2_4_old_content, file2_4_new_content);

    let file2_2_old_content = std::fs::read(fpath2_2).unwrap();
    let output = rsid3_run(&[OsStr::new("--force-id3v2.2"), fpath2_2]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let file2_2_new_content = std::fs::read(fpath2_2).unwrap();
    assert_eq!(file2_2_old_content, file2_2_new_content);

    let file2_3_old_content = std::fs::read(fpath2_3).unwrap();
    let output = rsid3_run(&[OsStr::new("--force-id3v2.3"), fpath2_3]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let file2_3_new_content = std::fs::read(fpath2_3).unwrap();
    assert_eq!(file2_3_old_content, file2_3_new_content);

    let file2_4_old_content = std::fs::read(fpath2_4).unwrap();
    let output = rsid3_run(&[OsStr::new("--force-id3v2.4"), fpath2_4]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let file2_4_new_content = std::fs::read(fpath2_4).unwrap();
    assert_eq!(file2_4_old_content, file2_4_new_content);
}

#[test]
fn converts_between_tag_versions_lossless() {
    let file = TestFile::id3v2_2();
    let fpath = file.path().as_os_str();

    let output = rsid3_run(&[OsStr::new("--TT2"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Sample Title");

    let output = rsid3_run(&[OsStr::new("--id3v2.3"), fpath]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());

    let output = rsid3_run(&[OsStr::new("--TIT2"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Sample Title");

    let output = rsid3_run(&[OsStr::new("--id3v2.4"), fpath]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());

    let output = rsid3_run(&[OsStr::new("--TIT2"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Sample Title");

    let output = rsid3_run(&[OsStr::new("--TMOO="), OsStr::new("Sample Mood"), fpath]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());

    let output = rsid3_run(&[OsStr::new("--id3v2.2"), fpath]);
    assert!(output.status.code().unwrap() == ExitCode::ActionFailed as i32);
    assert!(output.stdout.is_empty());

    let output = rsid3_run(&[OsStr::new("--TMOO-"), fpath]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());

    let output = rsid3_run(&[OsStr::new("--id3v2.2"), fpath]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());

    let output = rsid3_run(&[OsStr::new("--TT2"), fpath]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Sample Title");
}
