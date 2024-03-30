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
