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
use rsid3::*;
use rsid3::id3_helpers::*;
use rsid3::cli::{Cli, Action, PurgeOpt};
use id3::{Tag, TagLike, Version};

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
        let mut was_any_needed_frame_missing = false;

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
                        match print_tag_frame_query(&tag, &frame.to_id3_frame(), fpath) {
                            Ok(found) => {
                                if !found {
                                    was_any_needed_frame_missing = true;
                                }
                            },
                            Err(e) => {
                                eprintln!("rsid3: {e}");
                                return ExitCode::ActionFailed;
                            },
                        }
                    },
                    Action::Set(frame) => {
                        if let Err(e) = set_tag_frame(&mut tag, frame.to_id3_frame(), fpath) {
                            eprintln!("rsid3: {e}");
                            return ExitCode::ActionFailed;
                        }
                        tag_was_modified = true;
                    },
                    Action::Delete(frame) => {
                        match delete_tag_frame(&mut tag, &frame.to_id3_frame(), fpath) {
                            Ok(found) => {
                                tag_was_modified |= found;
                                if !found {
                                    was_any_needed_frame_missing = true;
                                }
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

        if was_any_needed_frame_missing {
            return ExitCode::FrameNotFound;
        }
    } else /* if cli.actions.is_empty() */ {
        if cli.files.is_empty() {
            Cli::print_usage();
            return ExitCode::BadArg;
        }

        // Print all frames if no options supplied
        let mut is_first = true;
        let mut any_tag_was_missing = false;
        for fpath in &cli.files {
            if is_first {
                is_first = false;
            } else {
                println!();
            }
            match print_all_file_frames_pretty(fpath) {
                Ok(found) => {
                    any_tag_was_missing |= !found;
                },
                Err(e) => {
                    eprintln!("rsid3: {e}");
                    return ExitCode::ActionFailed;
                },
            }
        }

        if any_tag_was_missing {
            return ExitCode::TagNotFound;
        }
    }

    ExitCode::Success
}
