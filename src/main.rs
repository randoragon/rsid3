mod cli;
mod id3_helpers;

use cli::Cli;
use id3_helpers::*;
use std::process::ExitCode;
use anyhow::{anyhow, Result};
use id3::{Tag, TagLike, Frame};

/// Prints frames from a file, with a custom delimiter.
fn print_file_frames(fpath: &str, frames: &Vec<Frame>, delimiter: &str) -> Result<()> {
    let tag = match Tag::read_from_path(fpath) {
        Ok(tag) => tag,
        Err(e) => return Err(anyhow!("Failed to read tags from file '{fpath}': {e}")),
    };

    let mut is_first = true;
    for frame in frames {
        match is_first {
            true => is_first = false,
            false => print!("{delimiter}"),
        }
        if let Err(e) = print_text_from_tag(&tag, frame) {
            eprintln!("rsid3: {e}");
        }
    }

    Ok(())
}

/// Deletes frames from a file.
fn delete_file_frames(fpath: &str, frames: &Vec<Frame>) -> Result<()> {
    let mut tag = match Tag::read_from_path(fpath) {
        Ok(tag) => tag,
        Err(e) => return Err(anyhow!("Failed to read tags from file '{fpath}': {e}")),
    };

    // Not the most efficient approach, but the id3 crate API is not the best either
    let mut was_modified = false;
    for frame in frames {
        let mut found = false;
        for removed_frame in tag.remove(frame.id()) {
            if frames_query_equal(frame, &removed_frame)? {
                // Remove this frame (i.e. don't add it back)
                found = true
            } else {
                tag.add_frame(removed_frame);
            }
        }
        let frame_str = match frame.id() {
            "WXXX" => format!("{}[{}]", frame.id(), get_content_wxxx(frame)?.description),
            "TXXX" => format!("{}[{}]", frame.id(), get_content_txxx(frame)?.description),
            "COMM" => {
                let comment = get_content_comm(frame)?;
                format!("{}[{}]({})", frame.id(), comment.description, comment.lang)
            },
            "USLT" => {
                let lyrics = get_content_uslt(frame)?;
                format!("{}[{}]({})", frame.id(), lyrics.description, lyrics.lang)
            },
            x => x.to_string(),
        };
        eprintln!("{fpath}: Could not delete {frame_str}: frame not found");
        was_modified |= found;
    }

    if was_modified {
        try_write_tag(&tag, &fpath, tag.version())?;
    }

    Ok(())
}

/// Pretty-prints all supported frames stored in the file.
fn print_all_file_frames_pretty(fpath: &str) -> Result<()> {
    let tag = match Tag::read_from_path(fpath) {
        Ok(tag) => tag,
        Err(e) => return Err(anyhow!("Failed to read tags from file '{fpath}': {e}")),
    };

    let n_frames = tag.frames().count();
    println!("{}: {}, {} frame{}:", fpath, tag.version(), n_frames,
        if n_frames == 1 { "" } else { "s" });
    for frame in tag.frames() {
        print_frame_pretty(frame)?;
    }

    Ok(())
}

// Writes frames into a file. Previous values are overwritten, if any.
fn set_file_frames(fpath: &str, frames: Vec<Frame>) -> Result<()> {
    let mut tag = match Tag::read_from_path(fpath) {
        Ok(tag) => tag,
        Err(e) => return Err(anyhow!("Failed to read tags from file '{fpath}': {e}")),
    };

    let mut was_modified = false;
    for frame in frames {
        match frame.id() {
            x if x.starts_with("T") || x.starts_with("W") || x == "COMM" || x == "USLT" => {
                let _ = tag.add_frame(frame);
                was_modified = true;
            },
            _ => return Err(anyhow!("Writing to {frame} is not supported")),
        }
    }

    if was_modified {
        try_write_tag(&tag, &fpath, tag_version)?;
    }

    Ok(())
}

fn main() -> ExitCode {
    let cli = match Cli::parse_args() {
        Ok(cli) => cli,
        Err(e) => {
            eprintln!("rsid3: {e}, try 'rsid3 --help'");
            return ExitCode::FAILURE;
        }
    };

    if cli.help {
        Cli::print_usage();
        return ExitCode::SUCCESS;
    }

    if cli.list_frames {
        Cli::print_all_frames();
        return ExitCode::SUCCESS;
    }

    // Define the delimiter
    if cli.delimiter.is_some() && cli.null_delimited {
        eprintln!("rsid3: --delimiter and --null-delimited options are mutually exclusive");
        return ExitCode::FAILURE;
    }
    let delimiter = if cli.null_delimited {
        '\0'.to_string()
    } else {
        cli.delimiter.clone().unwrap_or('\n'.to_string())
    };

    // Handle all get options
    for fpath in &cli.files {
        if let Err(e) = print_file_frames(fpath, &cli.get_frames, &delimiter) {
            eprintln!("rsid3: {e}");
            return ExitCode::FAILURE;
        }
    }

    // Handle all set options
    for fpath in &cli.files {
        if let Err(e) = set_file_frames(fpath, cli.set_frames.to_owned()) {
            eprintln!("rsid3: {e}");
            return ExitCode::FAILURE;
        }
    }

    // Handle all delete options
    for fpath in &cli.files {
        if let Err(e) = delete_file_frames(fpath, &cli.del_frames) {
            eprintln!("rsid3: {e}");
            return ExitCode::FAILURE;
        }
    }

    if cli.get_frames.is_empty() && cli.set_frames.is_empty() && cli.del_frames.is_empty() {
        if cli.files.is_empty() {
            Cli::print_usage();
            return ExitCode::FAILURE;
        }

        // Print all frames if no options supplied
        let mut is_first = true;
        for fpath in &cli.files {
            if is_first {
                is_first = false;
            } else {
                println!("");
            }
            if let Err(e) = print_all_file_frames_pretty(fpath) {
                eprintln!("rsid3: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
