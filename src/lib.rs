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
pub mod cli;
pub mod id3_helpers;
use crate::id3_helpers::*;
use crate::cli::ConvertOpt;
use std::path::Path;
use anyhow::{anyhow, Result};
use id3::{Tag, Version};

/// Exit codes used for various situations.
#[derive(Debug)]
pub enum ExitCode {
    /// Process finished successfully.
    Success = 0,

    /// Attempted read of a frame that does not exist in a file. This exit code is important,
    /// because it allows to determine that a frame does not exist, as opposed to it containing
    /// an empty string.
    FrameNotFound = 1,

    /// The user passed incorrect arguments.
    BadArg = 255,

    /// Executing some action failed.
    ActionFailed = 254,
}
impl std::process::Termination for ExitCode {
    fn report(self) -> std::process::ExitCode {
        match self {
            ExitCode::Success => std::process::ExitCode::SUCCESS,
            v => std::process::ExitCode::from(v as u8),
        }
    }
}

/// Pretty-prints all supported frames stored in the file.
pub fn print_all_file_frames_pretty(fpath: &impl AsRef<Path>) -> Result<()> {
    let tag = match Tag::read_from_path(fpath) {
        Ok(tag) => tag,
        Err(e) => match e.kind {
            id3::ErrorKind::NoTag => {
                eprintln!("{}: No tag found", fpath.as_ref().display());
                return Ok(());
            },
            _ => return Err(anyhow!("Failed to read tag from file '{}': {e}", fpath.as_ref().display())),
        }
    };

    let n_frames = tag.frames().count();
    println!("{}: {}, {} frame{}:", fpath.as_ref().display(), tag.version(), n_frames,
        if n_frames == 1 { "" } else { "s" });
    for frame in tag.frames() {
        print_frame_pretty(frame, tag.version())?;
    }

    Ok(())
}

/// Converts a tag according to the given command-line option.
/// On success, returns whether any conversion happened (`false` iff the tag's version was already
/// the same as the requested version).
pub fn convert_tag(tag: &mut Tag, opt: ConvertOpt) -> Result<bool> {
    let (tag_version, force) = match opt {
        ConvertOpt::Id3v22 => (Version::Id3v22, false),
        ConvertOpt::Id3v23 => (Version::Id3v23, false),
        ConvertOpt::Id3v24 => (Version::Id3v24, false),
        ConvertOpt::Id3v22Force => (Version::Id3v22, true),
        ConvertOpt::Id3v23Force => (Version::Id3v23, true),
        ConvertOpt::Id3v24Force => (Version::Id3v24, true),
    };
    if tag.version() == tag_version {
        return Ok(false);
    }
    *tag = tag_with_version_from(tag, tag_version, force)?;
    Ok(true)
}
