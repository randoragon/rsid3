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
use std::ffi::OsStr;
use std::path::Path;
use std::fs::{create_dir_all, copy};
use tempfile::NamedTempFile;
use std::process::{Command, Output};

/// Path to a sample MP3 file with no tags.
const SAMPLE_EMPTY: &str = "tests/samples/sample_0.mp3";
/// Path to a sample MP3 file with a TIT2 "Sample Title" tag.
const SAMPLE_TIT2: &str = "tests/samples/sample_TIT2.mp3";
/// Path to a sample MP3 file with a TXXX[Description] "Sample Content" tag.
const SAMPLE_TXXX: &str = "tests/samples/sample_TXXX.mp3";
/// Path to a sample MP3 file with a COMM[Description](eng) "Sample Content" tag.
const SAMPLE_COMM: &str = "tests/samples/sample_COMM.mp3";
/// Path to the directory for storing temporary files constructed and operated on in integration tests.
const SAMPLES_TMPDIR: &str = "tests/samples/tmp/";
/// Path to the rsid3 executable.
const PROGRAM_PATH: &str = "target/debug/rsid3";

pub struct TestFile {
    file: NamedTempFile,
}

/// Defines a new constructor of `TestFile` which creates a copy of a given sample file.
macro_rules! test_file_from_sample {
    ($func_name:ident, $fpath:path) => {
        pub fn $func_name() -> Self {
            create_dir_all(SAMPLES_TMPDIR).unwrap();
            let file = NamedTempFile::new_in(SAMPLES_TMPDIR).unwrap();
            copy($fpath, file.path()).unwrap();
            TestFile {
                file,
            }
        }
    };
}

impl TestFile {
    test_file_from_sample!(empty, SAMPLE_EMPTY);
    test_file_from_sample!(tit2, SAMPLE_TIT2);
    test_file_from_sample!(txxx, SAMPLE_TXXX);
    test_file_from_sample!(comm, SAMPLE_COMM);

    /// Returns the path to the test file.
    pub fn path(&self) -> &Path {
        self.file.path()
    }
}

pub fn rsid3_run(args: &[impl AsRef<OsStr>]) -> Output {
    let mut cmd = Command::new(PROGRAM_PATH);
    cmd.args(args);
    println!("Command: {:?}", cmd);
    let output = cmd.output().unwrap();
    println!("Status:  {:?}", output.status);
    println!("Stdout:  {:?}", String::from_utf8(output.stdout.clone()).unwrap());
    println!("Stderr:  {:?}", String::from_utf8(output.stderr.clone()).unwrap());
    output
}
