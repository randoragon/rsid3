mod cli;
mod id3_helpers;

use cli::{Cli, ConvertOpt};
use id3_helpers::*;
use std::process::ExitCode;
use anyhow::{anyhow, Result};
use id3::{Tag, TagLike, Frame, Version};

/// Prints frames from a file, with a custom delimiter.
fn print_file_frames(fpath: &str, frames: &Vec<Frame>, delimiter: &str) -> Result<()> {
    let tag = match Tag::read_from_path(fpath) {
        Ok(tag) => tag,
        Err(e) => match e.kind {
            id3::ErrorKind::NoTag => {
                eprintln!("rsid3: No tag found in '{fpath}'");
                return Ok(());
            },
            _ => return Err(anyhow!("Failed to read tags from file '{fpath}': {e}")),
        }
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
        Err(e) => match e.kind {
            id3::ErrorKind::NoTag => {
                eprintln!("rsid3: No tag found in '{fpath}'");
                return Ok(());
            },
            _ => return Err(anyhow!("Failed to read tags from file '{fpath}': {e}")),
        }
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
        Err(e) => match e.kind {
            id3::ErrorKind::NoTag => {
                eprintln!("rsid3: No tag found in '{fpath}'");
                return Ok(());
            },
            _ => return Err(anyhow!("Failed to read tags from file '{fpath}': {e}")),
        }
    };

    let n_frames = tag.frames().count();
    println!("{}: {}, {} frame{}:", fpath, tag.version(), n_frames,
        if n_frames == 1 { "" } else { "s" });
    for frame in tag.frames() {
        print_frame_pretty(frame)?;
    }

    Ok(())
}

/// Writes frames into a file. Previous values are overwritten, if any.
/// If `tag_version` is `None`, stick to the version in the existing tag.
/// `force` dictates whether to force the conversion (omit frames which cannot be converted),
/// or return an error if a lossless conversion is not possible.
fn set_file_frames(fpath: &str, frames: Vec<Frame>, tag_version: Option<Version>, force: bool) -> Result<()> {
    let mut tag = match Tag::read_from_path(fpath) {
        Ok(tag) => tag,
        Err(e) => match e.kind {
            id3::ErrorKind::NoTag => {
                eprintln!("rsid3: No tag found in '{fpath}'");
                return Ok(());
            },
            _ => return Err(anyhow!("Failed to read tags from file '{fpath}': {e}")),
        }
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

    if was_modified || tag_version.is_some_and(|v| v != tag.version()) {
        let tag_version = tag_version.unwrap();
        if force {
            tag = force_convert_tag(&tag, tag_version);
        }
        try_write_tag(&tag, &fpath, tag_version)?;
    }

    Ok(())
}

/// Forcefully convert a tag to a target ID3 version. Any frames that do not exist in the
/// target representation are simply omitted from the result.
fn force_convert_tag(tag: &Tag, target_version: Version) -> Tag {
    if tag.version() == target_version {
        return tag.clone();
    }

    let mut new_tag = Tag::with_version(target_version);
    for frame in tag.frames().filter(|x| x.id_for_version(target_version).is_some()) {
        new_tag.add_frame(frame.clone());
    }
    new_tag
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

    // Only one convertion option at a time is allowed
    if cli.convert_opts.len() > 1 {
        eprintln!("rsid3: it is illegal to pass multiple convert options");
        return ExitCode::FAILURE;
    }

    // Handle all get options
    for fpath in &cli.files {
        if let Err(e) = print_file_frames(fpath, &cli.get_frames, &delimiter) {
            eprintln!("rsid3: {e}");
            return ExitCode::FAILURE;
        }
    }

    // Establish which tag version to write, and whether to force a conversion
    let (tag_version, force_convert) = match cli.convert_opts.get(0) {
        Some(ConvertOpt::Id3v22) => (Some(Version::Id3v22), false),
        Some(ConvertOpt::Id3v23) => (Some(Version::Id3v23), false),
        Some(ConvertOpt::Id3v24) => (Some(Version::Id3v24), false),
        Some(ConvertOpt::Id3v22Force) => (Some(Version::Id3v22), true),
        Some(ConvertOpt::Id3v23Force) => (Some(Version::Id3v23), true),
        Some(ConvertOpt::Id3v24Force) => (Some(Version::Id3v24), true),
        None => (None, false),
    };

    // Handle all set options
    for fpath in &cli.files {
        if let Err(e) = set_file_frames(fpath, cli.set_frames.to_owned(), tag_version, force_convert) {
            eprintln!("rsid3: {e}");
            return ExitCode::FAILURE;
        }
    }

    // Handle the convert option. This may be superfluous, if the tag has already been converted
    // as a result of handling the set options above. But if no set options were passed, here is
    // where the conversion happens.
    if !cli.convert_opts.is_empty() {
        for fpath in &cli.files {
            if let Err(e) = set_file_frames(fpath, vec![], tag_version, force_convert) {
                eprintln!("rsid3: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    // Handle all delete options
    for fpath in &cli.files {
        if let Err(e) = delete_file_frames(fpath, &cli.del_frames) {
            eprintln!("rsid3: {e}");
            return ExitCode::FAILURE;
        }
    }

    if cli.get_frames.is_empty() && cli.set_frames.is_empty() && cli.del_frames.is_empty()
    && cli.convert_opts.is_empty() && cli.purge_opts.is_empty() {
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
