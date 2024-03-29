use anyhow::{anyhow, Result};
use id3::{Tag, TagLike, Frame, Version};
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

/// Get text contents from a tag, based on a frame query.
pub fn print_text_from_tag(tag: &Tag, frame: &Frame) -> Result<()> {
    match frame.id() {
        "TXXX" => {
            let desc_query = &get_content_txxx(frame)?.description;

            for txxx in tag.frames().filter(|&f| f.id() == "TXXX") {
                let extended_text = match get_content_txxx(txxx) {
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("rsid3: {e}");
                        continue;
                    },
                };
                if extended_text.description == *desc_query {
                    println!("{}", extended_text.value);
                    return Ok(());
                }
            }
            Err(anyhow!("TXXX frame with description '{desc_query}' not found"))
        },
        "WXXX" => {
            let desc_query = &get_content_wxxx(frame)?.description;
            for wxxx in tag.frames().filter(|&f| f.id() == "WXXX") {
                let extended_link = match get_content_wxxx(wxxx) {
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("rsid3: {e}");
                        continue;
                    },
                };
                if extended_link.description == *desc_query {
                    println!("{}", extended_link.link);
                    return Ok(());
                }
            }
            Err(anyhow!("WXXX frame with description '{desc_query}' not found"))
        },
        "COMM" => {
            let comment_query = get_content_comm(frame)?;
            let (desc_query, lang_query) = (&comment_query.description, &comment_query.lang);
            for comm in tag.frames().filter(|&f| f.id() == "COMM") {
                let comment = match get_content_comm(comm) {
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("rsid3: {e}");
                        continue;
                    },
                };
                if comment.description == *desc_query && (comment.lang == *lang_query || *lang_query == "first") {
                    println!("{}", comment.text);
                    return Ok(());
                }
            }
            Err(anyhow!("COMM frame with description '{desc_query}' and language '{lang_query}' not found"))
        },
        "USLT" => {
            let lyrics_query = get_content_uslt(frame)?;
            let (desc_query, lang_query) = (&lyrics_query.description, &lyrics_query.lang);
            for uslt in tag.frames().filter(|&f| f.id() == "USLT") {
                let lyrics = match get_content_uslt(uslt) {
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("rsid3: {e}");
                        continue;
                    },
                };
                if lyrics.description == *desc_query && (lyrics.lang == *lang_query || *lang_query == "first") {
                    println!("{}", lyrics.text);
                    return Ok(());
                }
            }
            Err(anyhow!("USLT frame with description '{desc_query}' and language '{lang_query}' not found"))
        },
        x if x.starts_with('T') => {
            let text_frame = match tag.get(x) {
                Some(frame) => frame,
                None => return Err(anyhow!("Frame not found: {x}")),
            };
            println!("{}", get_content_text(text_frame)?);
            Ok(())
        },
        x if x.starts_with('W') => {
            let link_frame = match tag.get(x) {
                Some(frame) => frame,
                None => return Err(anyhow!("Frame not found: {x}")),
            };
            println!("{}", get_content_link(link_frame)?);
            Ok(())
        },
        x => {
            let frame = match tag.get(x) {
                Some(frame) => frame,
                None => return Err(anyhow!("Frame not found: {x}")),
            };
            println!("{}", frame.content());
            Ok(())
        },
    }
}

/// Pretty-prints a single frame.
pub fn print_frame_pretty(frame: &Frame) -> Result<()> {
    match frame.id() {
        "TXXX" => {
            let extended_text = get_content_txxx(frame)?;
            println!("{}[{}]: {}", frame.id(), extended_text.description, extended_text.value);
        },
        "WXXX" => {
            let extended_link = get_content_wxxx(frame)?;
            println!("{}[{}]: {}", frame.id(), extended_link.description, extended_link.link);
        },
        "COMM" => {
            let comment = get_content_comm(frame)?;
            println!("{}[{}]({}): {}", frame.id(), comment.description, comment.lang, comment.text);
        },
        "USLT" => {
            let lyrics = get_content_uslt(frame)?;
            println!("{}[{}]({}): {}", frame.id(), lyrics.description, lyrics.lang, lyrics.text);
        },
        str if str.starts_with('T') => {
            println!("{}: {}", frame.id(), get_content_text(frame)?);
        },
        str if str.starts_with('W') => {
            println!("{}: {}", frame.id(), get_content_link(frame)?);
        },
        _ => {
            println!("{}: {}", frame.id(), frame.content());
        },
    }
    Ok(())
}

/// Returns whether two frames are identical except for the relevant content component.
/// E.g. two text types are equal iff their IDs match, but two COMMs are equal iff
/// their IDs, descriptions and languages match.
pub fn frames_query_equal(frame1: &Frame, frame2: &Frame) -> Result<bool, anyhow::Error> {
    if frame1.id() != frame2.id() {
        return Ok(false);
    }
    match frame1.id() {
        "TXXX" => {
            let extended_text1 = get_content_txxx(frame1)?;
            let extended_text2 = get_content_txxx(frame2)?;
            if extended_text1.description != extended_text2.description {
                return Ok(false);
            }
        },
        "WXXX" => {
            let extended_link1 = get_content_wxxx(frame1)?;
            let extended_link2 = get_content_wxxx(frame2)?;
            if extended_link1.description != extended_link2.description {
                return Ok(false);
            }
        },

        "COMM" => {
            let comment1 = get_content_comm(frame1)?;
            let comment2 = get_content_comm(frame2)?;
            if comment1.description != comment2.description || comment1.lang != comment2.lang {
                return Ok(false);
            }
        },
        "USLT" => {
            let lyrics1 = get_content_uslt(frame1)?;
            let lyrics2 = get_content_uslt(frame2)?;
            if lyrics1.description != lyrics2.description || lyrics1.lang != lyrics2.lang {
                return Ok(false);
            }
        },
        _ => (),
    }
    Ok(true)
}

/// Forcefully convert a tag to a target ID3 version. Any frames that do not exist in the
/// target representation are simply omitted from the result.
pub fn force_convert_tag(tag: &Tag, target_version: Version) -> Tag {
    if tag.version() == target_version {
        return tag.clone();
    }

    let mut new_tag = Tag::with_version(target_version);
    for frame in tag.frames().filter(|x| x.id_for_version(target_version).is_some()) {
        new_tag.add_frame(frame.clone());
    }
    new_tag
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
