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
mod cli;
mod id3_helpers;

use cli::{Cli, Action, ConvertOpt, PurgeOpt};
use std::path::Path;
use id3_helpers::*;
use anyhow::{anyhow, Result};
use id3::{Tag, TagLike, Version};

/// Exit codes used for various situations.
#[derive(Debug)]
enum ExitCode {
    /// Process finished successfully.
    Success = 0,

    /// The user passed incorrect arguments.
    BadArg = 1,

    /// Executing some action failed.
    ActionFailed = 2,
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
fn print_all_file_frames_pretty(fpath: &impl AsRef<Path>) -> Result<()> {
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
fn convert_tag(tag: &mut Tag, opt: ConvertOpt) -> Result<bool> {
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

fn main() -> ExitCode {
    let cli = match Cli::parse_args() {
        Ok(cli) => cli,
        Err(e) => {
            eprintln!("rsid3: {e}, try 'rsid3 --help'");
            return ExitCode::BadArg;
        }
    };

    if cli.help {
        Cli::print_usage();
        return ExitCode::Success;
    }

    if cli.version {
        Cli::print_version();
        return ExitCode::Success;
    }

    if cli.list_frames {
        Cli::print_all_frames();
        return ExitCode::Success;
    }

    // Define the separators
    if cli.frame_sep.is_some() && cli.frame_sep_null {
        eprintln!("rsid3: --frame-sep and --frame-sep-null options are mutually exclusive");
        return ExitCode::BadArg;
    }
    let frame_sep = if cli.frame_sep_null {
        '\0'.to_string()
    } else {
        cli.frame_sep.clone().unwrap_or('\n'.to_string())
    };
    if cli.file_sep.is_some() && cli.file_sep_null {
        eprintln!("rsid3: --file-sep and --file-sep-null options are mutually exclusive");
        return ExitCode::BadArg;
    }
    let file_sep = if cli.file_sep_null {
        '\0'.to_string()
    } else {
        cli.file_sep.clone().unwrap_or('\n'.to_string())
    };

    // Handle all actions
    if !cli.actions.is_empty() {
        let mut is_first_file_print = true;
        for fpath in &cli.files {
            // Read the file's tag
            let mut tag = match Tag::read_from_path(fpath) {
                Ok(tag) => tag,
                Err(e) => match e.kind {
                    id3::ErrorKind::NoTag => {
                        Tag::with_version(Version::Id3v24)
                    },
                    _ => {
                        eprintln!("rsid3: Failed to read tag from file '{fpath}': {e}");
                        break;
                    },
                }
            };
            let mut tag_was_modified = false;
            let mut is_first_frame_print = true;

            for action in &cli.actions {
                if let Err(e) = action.is_supported(&tag) {
                    eprintln!("rsid3: {e}");
                }
                match action {
                    Action::Print(frame) => {
                        if !is_first_frame_print {
                            print!("{frame_sep}");
                        } else {
                            is_first_frame_print = false;
                            if !is_first_file_print {
                                print!("{file_sep}");
                            } else {
                                is_first_file_print = false;
                            }
                        }
                        if let Err(e) = print_tag_frame_query(&tag, &frame.to_id3_frame(), fpath) {
                            eprintln!("rsid3: {e}");
                            return ExitCode::ActionFailed;
                        }
                    },
                    Action::Set(frame) => {
                        let _ = tag.add_frame(frame.to_id3_frame());
                        tag_was_modified = true;
                    },
                    Action::Delete(frame) => {
                        match delete_tag_frame(&mut tag, &frame.to_id3_frame(), fpath) {
                            Ok(modified) => {
                                tag_was_modified |= modified;
                            },
                            Err(e) => {
                                eprintln!("rsid3: {e}");
                                return ExitCode::ActionFailed;
                            },
                        }
                    },
                    Action::Convert(opt) => {
                        match convert_tag(&mut tag, *opt) {
                            Ok(modified) => {
                                tag_was_modified |= modified;
                            },
                            Err(e) => {
                                eprintln!("rsid3: {e}");
                                return ExitCode::ActionFailed;
                            },
                        }
                    },
                    Action::Purge(opt) => {
                        if match opt {
                            PurgeOpt::Id3v22 => tag.version() == Version::Id3v22,
                            PurgeOpt::Id3v23 => tag.version() == Version::Id3v23,
                            PurgeOpt::Id3v24 => tag.version() == Version::Id3v24,
                            PurgeOpt::All => true,
                        } {
                            match id3::v1v2::remove_from_path(fpath) {
                                Ok(_) => {
                                    tag = Tag::with_version(Version::Id3v24);
                                    tag_was_modified = false;
                                },
                                Err(e) => {
                                    eprintln!("rsid3: Failed to purge the tag of '{fpath}': {e}");
                                },
                            }
                        }
                    },
                }
            }

            // Write the tag back to the file, if it was modified
            if tag_was_modified {
                if let Err(e) = try_write_tag(&tag, &fpath, tag.version()) {
                    eprintln!("rsid3: {e}");
                    return ExitCode::ActionFailed;
                }
            }
        }
    } else /* if cli.actions.is_empty() */ {
        if cli.files.is_empty() {
            Cli::print_usage();
            return ExitCode::BadArg;
        }

        // Print all frames if no options supplied
        let mut is_first = true;
        for fpath in &cli.files {
            if is_first {
                is_first = false;
            } else {
                println!();
            }
            if let Err(e) = print_all_file_frames_pretty(fpath) {
                eprintln!("rsid3: {e}");
                return ExitCode::ActionFailed;
            }
        }
    }

    ExitCode::Success
}
