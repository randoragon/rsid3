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
use anyhow::{anyhow, Result};
use id3::{Tag, TagLike, Frame, Version, Content};
use id3::frame::{Comment, Lyrics, ExtendedText, ExtendedLink};
use std::io::empty;
use std::path::Path;

/// Convenience wrapper for getting any simple text content.
pub fn get_content_text(frame: &Frame) -> Result<&str> {
    match frame.content().text() {
        Some(x) => Ok(x),
        None => Err(anyhow!("Frame claims to be {} with T but has no text content: {frame:?}", frame.id())),
    }
}

/// Convenience wrapper for getting any link content.
pub fn get_content_link(frame: &Frame) -> Result<&str> {
    match frame.content().link() {
        Some(x) => Ok(x),
        None => Err(anyhow!("Frame claims to be {} with T but has no link content: {frame:?}", frame.id())),
    }
}

/// Convenience wrapper for getting TXXX content.
pub fn get_content_txxx(frame: &Frame) -> Result<&ExtendedText> {
    match frame.content().extended_text() {
        Some(x) => Ok(x),
        None => Err(anyhow!("Frame claims to be TXXX but has no extended text content: {frame:?}")),
    }
}

/// Convenience wrapper for getting WXXX content.
pub fn get_content_wxxx(frame: &Frame) -> Result<&ExtendedLink> {
    match frame.content().extended_link() {
        Some(x) => Ok(x),
        None => Err(anyhow!("Frame claims to be WXXX but has no extended link content: {frame:?}")),
    }
}

/// Convenience wrapper for getting COMM content.
pub fn get_content_comm(frame: &Frame) -> Result<&Comment> {
    match frame.content().comment() {
        Some(x) => Ok(x),
        None => Err(anyhow!("Frame claims to be COMM but has no comment content: {frame:?}")),
    }
}

/// Convenience wrapper for getting USLT content.
pub fn get_content_uslt(frame: &Frame) -> Result<&Lyrics> {
    match frame.content().lyrics() {
        Some(x) => Ok(x),
        None => Err(anyhow!("Frame claims to be USLT but has no lyrics content: {frame:?}")),
    }
}

/// Returns a string representation of a frame, WITHOUT CONTENT.
pub fn frame_to_string(frame: &Frame) -> Result<String, anyhow::Error> {
    let string = match frame.id() {
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
    Ok(string)
}

/// Attempts to find a tag frame matching a query and prints its contents as text.
/// `fpath` is only used for message prints.
/// Returns whether a frame was found and printed.
pub fn print_tag_frame_query(tag: &Tag, query: &Frame, fpath: impl AsRef<Path>) -> Result<bool> {
    for frame in tag.frames() {
        if frame_matches_query(frame, query)? {
            match query.id() {
                "TXXX" => {
                    print!("{}", get_content_txxx(frame)?.value);
                    return Ok(true);
                },
                "WXXX" => {
                    print!("{}", get_content_wxxx(frame)?.link);
                    return Ok(true);
                },
                "COMM" => {
                    print!("{}", get_content_comm(frame)?.text);
                    return Ok(true);
                },
                "USLT" => {
                    print!("{}", get_content_uslt(frame)?.text);
                    return Ok(true);
                },
                x if x.starts_with('T') && x != "TIPL" => {
                    print!("{}", get_content_text(frame)?);
                    return Ok(true);
                },
                x if x.starts_with('W') => {
                    print!("{}", get_content_link(frame)?);
                    return Ok(true);
                },
                _ => {
                    print!("{}", frame.content());
                    return Ok(true);
                },
            }
        }
    }
    // Frame not found
    eprintln!("{}: Could not print {}: Frame not found", fpath.as_ref().display(), frame_to_string(query)?);
    Ok(false)
}

/// Pretty-prints a single frame's name and contents.
pub fn print_frame_pretty(frame: &Frame, version: Version) -> Result<()> {
    let id = match frame.id_for_version(version) {
        Some(x) => x,
        _ => return Err(anyhow!("Cannot obtain ID for the specified version")),
    };
    match frame.id() {
        "TXXX" => {
            let extended_text = get_content_txxx(frame)?;
            println!("{}[{}]: {}", id, extended_text.description, extended_text.value);
        },
        "WXXX" => {
            let extended_link = get_content_wxxx(frame)?;
            println!("{}[{}]: {}", id, extended_link.description, extended_link.link);
        },
        "COMM" => {
            let comment = get_content_comm(frame)?;
            println!("{}[{}]({}): {}", id, comment.description, comment.lang, comment.text);
        },
        "USLT" => {
            let lyrics = get_content_uslt(frame)?;
            println!("{}[{}]({}): {}", id, lyrics.description, lyrics.lang, lyrics.text);
        },
        str if str.starts_with('T') && str != "TIPL" => {
            println!("{}: {}", id, get_content_text(frame)?);
        },
        str if str.starts_with('W') => {
            println!("{}: {}", id, get_content_link(frame)?);
        },
        _ => {
            println!("{}: {}", id, frame.content());
        },
    }
    Ok(())
}

/// Sets a frame in a tag.
/// Responds correctly to lang set to "first" in frames that support it.
pub fn set_tag_frame(tag: &mut Tag, mut frame: Frame, fpath: impl AsRef<Path>) -> Result<()> {
    let overwrite_first = match frame.id() {
        "COMM" => get_content_comm(&frame)?.lang == "first",
        "USLT" => get_content_uslt(&frame)?.lang == "first",
        _ => false,
    };
    if overwrite_first {
        // Ensure a matching frame exists -- if not, abort
        let mut found = false;
        for tag_frame in tag.frames() {
            if frame_matches_query(tag_frame, &frame)? {
                // Create identical copies of frame, but with LANG updated to the found value.
                // This is a bit ugly, but it's the easiest and simplest way to do this.
                frame = match tag_frame.id() {
                    "COMM" => {
                        let found_content = get_content_comm(tag_frame)?;
                        let new_content = get_content_comm(&frame)?;
                        assert!(found_content.description == new_content.description);
                        let comment = Comment {
                            description: found_content.description.clone(),
                            lang: found_content.lang.clone(),
                            text: new_content.text.clone(),
                        };
                        Frame::with_content("COMM", Content::Comment(comment))
                    },
                    "USLT" => {
                        let found_content = get_content_uslt(tag_frame)?;
                        let new_content = get_content_uslt(&frame)?;
                        assert!(found_content.description == new_content.description);
                        let lyrics = Lyrics {
                            description: found_content.description.clone(),
                            lang: found_content.lang.clone(),
                            text: new_content.text.clone(),
                        };
                        Frame::with_content("USLT", Content::Lyrics(lyrics))
                    },
                    _ => panic!("internal logic error for {frame:?} and {tag_frame:?}"),
                };
                found = true;
                break;
            }
        }
        if !found {
            eprintln!("{}: Could not set {}: LANG set to \"first\", yet no matching frame exists", fpath.as_ref().display(), frame_to_string(&frame)?);
            return Ok(())
        }
    }

    let _ = tag.add_frame(frame);
    Ok(())
}

/// Deletes a frame matching a query from a tag.
/// `fpath` is only used for message prints.
/// Returns whether the frame was found and deleted.
pub fn delete_tag_frame(tag: &mut Tag, query: &Frame, fpath: impl AsRef<Path>) -> Result<bool> {
    let mut found = false;

    let only_remove_first_match = match query.id() {
        "COMM" => get_content_comm(query)?.lang == "first",
        "USLT" => get_content_uslt(query)?.lang == "first",
        _ => false,
    };
    let mut ignore_remaining_matches = false;

    // Not the most efficient approach, but the id3 crate does not seem to provide a nicer way
    for removed_frame in tag.remove(query.id()) {
        if !ignore_remaining_matches && frame_matches_query(&removed_frame, query)? {
            // Remove this frame (i.e. don't add it back)
            found = true;
            if only_remove_first_match {
                ignore_remaining_matches = true;
            }
        } else {
            tag.add_frame(removed_frame);
        }
    }
    if !found {
        eprintln!("{}: Could not delete {}: Frame not found", fpath.as_ref().display(), frame_to_string(query)?);
    }
    Ok(found)
}

/// Returns whether two frames are identical except for the relevant content component.
/// E.g. two text types are equal iff their IDs match, but two COMMs are equal iff
/// their IDs, descriptions and languages match.
pub fn frame_matches_query(frame: &Frame, query: &Frame) -> Result<bool, anyhow::Error> {
    if frame.id() != query.id() {
        return Ok(false);
    }
    match frame.id() {
        "TXXX" => {
            let extended_text_f = get_content_txxx(frame)?;
            let extended_text_q = get_content_txxx(query)?;
            if extended_text_f.description != extended_text_q.description {
                return Ok(false);
            }
        },
        "WXXX" => {
            let extended_link_f = get_content_wxxx(frame)?;
            let extended_link_q = get_content_wxxx(query)?;
            if extended_link_f.description != extended_link_q.description {
                return Ok(false);
            }
        },

        "COMM" => {
            let comment_f = get_content_comm(frame)?;
            let comment_q = get_content_comm(query)?;
            if comment_f.description != comment_q.description || (comment_f.lang != comment_q.lang && comment_q.lang != "first") {
                return Ok(false);
            }
        },
        "USLT" => {
            let lyrics_f = get_content_uslt(frame)?;
            let lyrics_q = get_content_uslt(query)?;
            println!("comment_q.lang = {}", lyrics_q.lang);
            if lyrics_f.description != lyrics_q.description || (lyrics_f.lang != lyrics_q.lang && lyrics_q.lang != "first") {
                return Ok(false);
            }
        },
        _ => (),
    }
    Ok(true)
}

/// Create a new tag of the given version, from an existing tag.
/// If `force` is true, any frames that cannot exist in the target version are simply omitted from
/// the result. Otherwise, an error is returned.
pub fn tag_with_version_from(tag: &Tag, target_version: Version, force: bool) -> Result<Tag> {
    if tag.version() == target_version {
        return Ok(tag.clone());
    }

    let mut new_tag = Tag::with_version(target_version);
    if force {
        for frame in tag.frames().filter(|x| x.id_for_version(target_version).is_some()) {
            new_tag.add_frame(frame.clone());
        }
    } else {
        let incompatible_frames = tag.frames()
            .filter(|&x| x.id_for_version(target_version).is_none())
            .map(|x| x.id())
            .collect::<Vec<&str>>();
        if !incompatible_frames.is_empty() {
            return Err(anyhow!("Cannot convert tag from {} to {}: Incompatible frames: {}",
                tag.version(), target_version, incompatible_frames.join(", ")));
        }
        for frame in tag.frames() {
            new_tag.add_frame(frame.clone());
        }
    }
    Ok(new_tag)
}

/// Attempt to write a tag to a file. `Tag.write_to_path()` does this, but it has the side-effect
/// of deleting the tag from the target file in case of failure. This function is a wrapper that
/// first tries to write the tag to an `std::io::Empty` dummy file, and will update the real file
/// only if that trial write succeeded.
pub fn try_write_tag(tag: &Tag, fpath: &impl AsRef<Path>, version: Version) -> Result<()> {
    if let Err(e) = tag.write_to(empty(), version) {
        return Err(anyhow!("Failed to compose tag of '{}': {e}", fpath.as_ref().display()));
    }
    if let Err(e) = tag.write_to_path(fpath, version) {
        // All errors caused by tag formats should have been caught in the previous if block.
        // This should ideally only catch errors related to OS-level failures, e.g. insufficient
        // storage, invalid path, etc.
        return Err(anyhow!("Failed to write tag to '{}': {e}", fpath.as_ref().display()));
    }
    Ok(())
}
