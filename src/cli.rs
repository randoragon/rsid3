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
use std::env::args;
use anyhow::{anyhow, Result};
use id3::{Frame, Content};
use id3::frame::{Comment, Lyrics, ExtendedText, ExtendedLink};

/// Represents all options passed to the program on the command line.
#[derive(Debug)]
pub struct Cli {
    pub help: bool,
    pub version: bool,
    pub list_frames: bool,
    pub frame_sep: Option<String>,
    pub file_sep: Option<String>,
    pub frame_sep_null: bool,
    pub file_sep_null: bool,
    pub actions: Vec<Action>,
    pub files: Vec<String>,
}

/// Represents a single action passed by the user on the command line.
#[derive(Debug)]
pub enum Action {
    Print(Frame),
    Set(Frame),
    Delete(Frame),
    Convert(ConvertOpt),
    Purge(PurgeOpt),
}

/// Represents one of convert options passed to the program on the command line.
#[derive(Debug, Copy, Clone)]
pub enum ConvertOpt {
    Id3v22,
    Id3v23,
    Id3v24,
    Id3v22Force,
    Id3v23Force,
    Id3v24Force,
}

/// Represents one of purge options passed to the program on the command line.
#[derive(Debug, Copy, Clone)]
pub enum PurgeOpt {
    Id3v22,
    Id3v23,
    Id3v24,
    All,
}

impl Cli {
    /// Prints how to use the program.
    pub fn print_usage() {
        println!("Usage:  rsid3 [OPTION] [--] FILE...");
        println!();
        println!("Reads or writes ID3v2 tags in mp3 files.");
        println!("Supported standards: ID3v2.2, ID3v2.3, ID3v2.4.");
        println!();
        println!("Options:");
        println!("  -h, --help               Show this help and exit.");
        println!("  -V, --version            Print version information.");
        println!("  -L, --list-frames        List all supported frames.");
        println!("  -d SEP, --frame-sep SEP  Separate printed frames with SEP (default: \\n).");
        println!("  -D SEP, --file-sep SEP   Separate printed files with SEP (default: \\n).");
        println!("  -0d, --frame-sep-null    Separate printed frames with the null byte.");
        println!("  -0D, --file-sep-null     Separate printed files with the null byte.");
        println!();
        println!("  --FRAME                  Print the value of FRAME.");
        println!("  --FRAME DESC             Print the value of FRAME (TXXX, WXXX).");
        println!("  --FRAME DESC LANG        Print the value of FRAME (COMM, USLT).");
        println!("  --FRAME= TEXT            Set the value of FRAME.");
        println!("  --FRAME= DESC TEXT       Set the value of FRAME (TXXX, WXXX).");
        println!("  --FRAME= DESC LANG TEXT  Set the value of FRAME (COMM, USLT).");
        println!("  --FRAME-                 Delete FRAME.");
        println!("  --FRAME- DESC            Delete FRAME (TXXX, WXXX).");
        println!("  --FRAME- DESC LANG       Delete FRAME (COMM, USLT).");
        println!();
        println!("  --id3v2.2                Convert tags to ID3v2.2 (lossless; may fail).");
        println!("  --id3v2.3                Convert tags to ID3v2.3 (lossless; may fail).");
        println!("  --id3v2.4                Convert tags to ID3v2.4 (lossless; may fail).");
        println!("  --force-id3v2.2          Convert tags to ID3v2.2 (omit non-convertible frames; always succeeds).");
        println!("  --force-id3v2.3          Convert tags to ID3v2.3 (omit non-convertible frames; always succeeds).");
        println!("  --force-id3v2.4          Convert tags to ID3v2.4 (omit non-convertible frames; always succeeds).");
        println!("  --purge-id3v2.2          Purge ID3v2.2 tags, if present.");
        println!("  --purge-id3v2.3          Purge ID3v2.3 tags, if present.");
        println!("  --purge-id3v2.4          Purge ID3v2.4 tags, if present.");
        println!("  --purge-all              Purge all ID3v2 tags, if present.");
        println!();
        println!("If the value of LANG is irrelevant when printing a frame, 'first'");
        println!("can be passed instead, in which case the first frame with a matching");
        println!("DESC is printed.");
        println!();
        println!("If no print/set/delete/convert/purge options are passed, all frames are printed.");
        println!("Any number of print/set/delete/convert/purge options can be passed in any order.");
        println!("The options are executed in the same order as they were passed in. This allows");
        println!("for chaining many operations under a single command.");
        println!("If no convert options are passed, rsid3 keeps the existing tag versions,");
        println!("or defaults to ID3v2.4 when creating new tags from scratch.");
    }

    /// Prints the current version of rsid3.
    pub fn print_version() {
        println!("rsid3 {}+{}\nBuilt on {}",
            env!("CARGO_PKG_VERSION"),
            env!("VERGEN_GIT_SHA").chars().take(8).collect::<String>(),
            env!("VERGEN_BUILD_TIMESTAMP"));
    }

    /// Prints the available frames.
    pub fn print_all_frames() {
        println!("Read-write frames:");
        println!("COMM	User comment (DESC, LANG, TEXT)");
        println!("TALB	Album");
        println!("TBPM	Beats per minute");
        println!("TCAT	iTunes podcast category");
        println!("TCMP	iTunes compilation flag");
        println!("TCOM	Composer");
        println!("TCON	Content type (genre)");
        println!("TCOP	Copyright");
        println!("TDAT	Date of recording (DDMM)");
        println!("TDEN	Encoding time");
        println!("TDES	iTunes podcast description");
        println!("TDLY	Audio delay (ms)");
        println!("TDOR	Original release time");
        println!("TDRC	Recording time");
        println!("TDRL	Release time");
        println!("TDTG	Tagging time");
        println!("TENC	Encoder");
        println!("TEXT	Lyricist");
        println!("TFLT	File type");
        println!("TGID	iTunes podcast identifier");
        println!("TIME	Time of recording (HHMM)");
        println!("TIPL	Involved people list");
        println!("TIT1	Content group description");
        println!("TIT2	Title");
        println!("TIT3	Subtitle/description refinement");
        println!("TKEY	Starting key");
        println!("TKWD	iTunes podcast keywords");
        println!("TLAN	Audio languages");
        println!("TLEN	Audio length (ms)");
        println!("TMCL	Musicians credits list");
        println!("TMED	Source media type");
        println!("TMOO	Mood");
        println!("TOAL	Original album");
        println!("TOFN	Original filename");
        println!("TOLY	Original lyricist");
        println!("TOPE	Original artist/performer");
        println!("TORY	Original release year");
        println!("TOWN	Owner/Licensee");
        println!("TPE1	Lead artist/performer/soloist/group");
        println!("TPE2	Band/Orchestra/Accompaniment");
        println!("TPE3	Conductor");
        println!("TPE4	Interpreter/Remixer/Modifier");
        println!("TPOS	Part of set");
        println!("TPRO	Produced");
        println!("TPUB	Publisher");
        println!("TRCK	Track number");
        println!("TRDA	Recording dates");
        println!("TRSN	Internet radio station name");
        println!("TRSO	Internet radio station owner");
        println!("TSIZ	Size of audio data (bytes)");
        println!("TSO2	iTunes album artist sort");
        println!("TSOA	Album sort order key");
        println!("TSOC	iTunes composer sort");
        println!("TSOP	Performer sort order key");
        println!("TSOT	Title sort order key");
        println!("TSRC	International Standard Recording Code (ISRC)");
        println!("TSSE	Encoder settings");
        println!("TSST	Set subtitle");
        println!("TXXX	User-defined text data (DESC, TEXT)");
        println!("TYER	Year of recording");
        println!("USLT	Unsynchronised lyrics/text transcription (DESC, LANG, TEXT)");
        println!("WCOM	Commercial information");
        println!("WCOP	Copyright information");
        println!("WFED	iTunes podcast feed");
        println!("WOAF	Official file information");
        println!("WOAR	Official artist/performer information");
        println!("WOAS	Official source information");
        println!("WORS	Official internet radio information");
        println!("WPAY	Payment information");
        println!("WPUB	Official publisher information");
        println!("WXXX	User-defined URL data (DESC, URL)");
        println!();
        println!("Read-only frames (rudimentary support):");
        println!("AENC	Audio encryption");
        println!("APIC	Attached (or linked) picture");
        println!("ASPI	Audio seek point index");
        println!("CHAP	Chapter");
        println!("COMR	Commercial frame");
        println!("CTOC	Table of contents");
        println!("ENCR	Encryption method registration");
        println!("EQU2	Equalization 2");
        println!("ETCO	Event timing codes");
        println!("GEOB	General encapsulated object");
        println!("GRID	Group identification registration");
        println!("GRP1	iTunes grouping");
        println!("IPLS	Involved people list");
        println!("LINK	Linked information");
        println!("MCDI	Binary dump of CD's TOC");
        println!("MLLT	MPEG location lookup table");
        println!("MVIN	iTunes movement number/count");
        println!("MVNM	iTunes movement name");
        println!("OWNE	Ownership frame");
        println!("PCNT	Play counter");
        println!("PCST	iTunes podcast flag");
        println!("POPM	Popularimeter");
        println!("POSS	Position synchronisation frame");
        println!("PRIV	Private frame");
        println!("RBUF	Recommended buffer size");
        println!("RVA2	Relative volume adjustment 2");
        println!("RVAD	Relative volume adjustment");
        println!("RVRB	Reverb");
        println!("SEEK	Seek frame");
        println!("SIGN	Signature frame");
        println!("SYLT	Synchronised lyrics/text");
        println!("SYTC	Synchronised tempo codes");
        println!("UFID	Unique file identifier");
        println!("USER	Terms of use");
    }

    /// Construct a Cli object representing passed command-line arguments.
    pub fn parse_args() -> Result<Self> {
        let args: Vec<String> = args().collect();
        let mut help = false;
        let mut version = false;
        let mut list_frames = false;
        let mut frame_sep: Option<String> = None;
        let mut file_sep: Option<String> = None;
        let mut frame_sep_null = false;
        let mut file_sep_null = false;
        let mut actions = vec![];
        let mut i = 1;
        while i < args.len() {
            let arg = args[i].as_str();
            match arg {
                "-h" | "--help" => { help = true; },
                "-V" | "--version" => { version = true; },
                "-L" | "--list-frames" => { list_frames = true; },
                "-d" | "--frame-sep" => {
                    if i + 1 >= args.len() {
                        return Err(anyhow!("1 argument expected after --frame-sep"));
                    }
                    frame_sep = Some(args[i + 1].clone());
                    i += 1;
                },
                str if str.starts_with("-d") => {
                    frame_sep = Some(((args[i])[2..]).to_string());
                },
                "-D" | "--file-sep" => {
                    if i + 1 >= args.len() {
                        return Err(anyhow!("1 argument expected after --file-sep"));
                    }
                    file_sep = Some(args[i + 1].clone());
                    i += 1;
                },
                str if str.starts_with("-D") => {
                    file_sep = Some(((args[i])[2..]).to_string());
                },
                "-0d" | "--frame-sep-null" => { frame_sep_null = true; },
                "-0D" | "--file-sep-null" => { file_sep_null = true; },
                "--" => { i += 1; break; },

                "--COMM" => {
                    if i + 2 >= args.len() {
                        return Err(anyhow!("2 arguments expected after --COMM"));
                    }
                    let comment = Comment {
                        description: args[i + 1].clone(),
                        lang: args[i + 2].clone(),
                        text: "".to_string(),
                    };
                    actions.push(Action::Print(Frame::with_content("COMM", Content::Comment(comment))));
                    i += 2;
                }
                "--USLT" => {
                    if i + 2 >= args.len() {
                        return Err(anyhow!("2 arguments expected after --USLT"));
                    }
                    let lyrics = Lyrics {
                        description: args[i + 1].clone(),
                        lang: args[i + 2].clone(),
                        text: "".to_string(),
                    };
                    actions.push(Action::Print(Frame::with_content("USLT", Content::Lyrics(lyrics))));
                    i += 2;
                },

                "--TXXX" => {
                    if i + 1 >= args.len() {
                        return Err(anyhow!("1 argument expected after --TXXX"));
                    }
                    let extended_text = ExtendedText {
                        value: "".to_string(),
                        description: args[i + 1].clone(),
                    };
                    actions.push(Action::Print(Frame::with_content("TXXX", Content::ExtendedText(extended_text))));
                    i += 1;
                },
                "--WXXX" => {
                    if i + 1 >= args.len() {
                        return Err(anyhow!("1 argument expected after --WXXX"));
                    }
                    let extended_link = ExtendedLink {
                        link: "".to_string(),
                        description: args[i + 1].clone(),
                    };
                    actions.push(Action::Print(Frame::with_content("WXXX", Content::ExtendedLink(extended_link))));
                    i += 1;
                },

                // All parameterless getters
                str if Cli::is_getter_arg(str) => {
                    actions.push(Action::Print(Frame::text(&str[2..], "")));
                },

                "--COMM=" => {
                    if i + 3 >= args.len() {
                        return Err(anyhow!("3 arguments expected after --COMM="));
                    }
                    let comment = Comment {
                        description: args[i + 1].clone(),
                        lang: args[i + 2].clone(),
                        text: args[i + 3].clone(),
                    };
                    actions.push(Action::Set(Frame::with_content("COMM", Content::Comment(comment))));
                    i += 3;
                }
                "--USLT=" => {
                    if i + 3 >= args.len() {
                        return Err(anyhow!("3 arguments expected after --USLT="));
                    }
                    let lyrics = Lyrics {
                        description: args[i + 1].clone(),
                        lang: args[i + 2].clone(),
                        text: args[i + 3].clone(),
                    };
                    actions.push(Action::Set(Frame::with_content("USLT", Content::Lyrics(lyrics))));
                    i += 3;
                }

                "--TXXX=" => {
                    if i + 2 >= args.len() {
                        return Err(anyhow!("2 arguments expected after --TXXX="));
                    }
                    let extended_text = ExtendedText {
                        description: args[i + 1].clone(),
                        value: args[i + 2].clone(),
                    };
                    actions.push(Action::Set(Frame::with_content("TXXX", Content::ExtendedText(extended_text))));
                    i += 2;
                },
                "--WXXX=" => {
                    if i + 2 >= args.len() {
                        return Err(anyhow!("2 arguments expected after --WXXX="));
                    }
                    let extended_link = ExtendedLink {
                        description: args[i + 1].clone(),
                        link: args[i + 2].clone(),
                    };
                    actions.push(Action::Set(Frame::with_content("WXXX", Content::ExtendedLink(extended_link))));
                    i += 2;
                },

                // All parameterless setters
                str if Cli::is_setter_arg(str) => {
                    if i + 1 >= args.len() {
                        return Err(anyhow!("1 argument expected after {str}"));
                    }
                    let text = args[i + 1].clone();
                    actions.push(Action::Set(Frame::text(&str[2..(str.len() - 1)], text)));
                    i += 1;
                },

                "--COMM-" => {
                    if i + 2 >= args.len() {
                        return Err(anyhow!("2 arguments expected after --COMM"));
                    }
                    let comment = Comment {
                        description: args[i + 1].clone(),
                        lang: args[i + 2].clone(),
                        text: "".to_string(),
                    };
                    actions.push(Action::Delete(Frame::with_content("COMM", Content::Comment(comment))));
                    i += 2;
                }
                "--USLT-" => {
                    if i + 2 >= args.len() {
                        return Err(anyhow!("2 arguments expected after --USLT"));
                    }
                    let lyrics = Lyrics {
                        description: args[i + 1].clone(),
                        lang: args[i + 2].clone(),
                        text: "".to_string(),
                    };
                    actions.push(Action::Delete(Frame::with_content("USLT", Content::Lyrics(lyrics))));
                    i += 2;
                },

                "--TXXX-" => {
                    if i + 1 >= args.len() {
                        return Err(anyhow!("1 argument expected after --TXXX"));
                    }
                    let extended_text = ExtendedText {
                        value: "".to_string(),
                        description: args[i + 1].clone(),
                    };
                    actions.push(Action::Delete(Frame::with_content("TXXX", Content::ExtendedText(extended_text))));
                    i += 1;
                },
                "--WXXX-" => {
                    if i + 1 >= args.len() {
                        return Err(anyhow!("1 argument expected after --WXXX"));
                    }
                    let extended_link = ExtendedLink {
                        link: "".to_string(),
                        description: args[i + 1].clone(),
                    };
                    actions.push(Action::Delete(Frame::with_content("WXXX", Content::ExtendedLink(extended_link))));
                    i += 1;
                },

                // All parameterless delete args
                str if Cli::is_delete_arg(str) => {
                    actions.push(Action::Delete(Frame::text(&str[2..(str.len() - 1)], "")));
                },

                "--id3v2.2" => {
                    actions.push(Action::Convert(ConvertOpt::Id3v22));
                },
                "--id3v2.3" => {
                    actions.push(Action::Convert(ConvertOpt::Id3v23));
                },
                "--id3v2.4" => {
                    actions.push(Action::Convert(ConvertOpt::Id3v24));
                },

                "--force-id3v2.2" => {
                    actions.push(Action::Convert(ConvertOpt::Id3v22Force));
                },
                "--force-id3v2.3" => {
                    actions.push(Action::Convert(ConvertOpt::Id3v23Force));
                },
                "--force-id3v2.4" => {
                    actions.push(Action::Convert(ConvertOpt::Id3v24Force));
                },

                "--purge-id3v2.2" => {
                    actions.push(Action::Purge(PurgeOpt::Id3v22));
                },
                "--purge-id3v2.3" => {
                    actions.push(Action::Purge(PurgeOpt::Id3v23));
                },
                "--purge-id3v2.4" => {
                    actions.push(Action::Purge(PurgeOpt::Id3v24));
                },
                "--purge-all" => {
                    actions.push(Action::Purge(PurgeOpt::All));
                },

                str => {
                    if str.starts_with('-') {
                        return Err(anyhow!("Unknown option: '{arg}'"));
                    }
                    break;
                }
            };
            i += 1;
        }

        let files = (i..args.len())
            .map(|x| args[x].clone())
            .collect();

        Ok(Cli {
            help,
            version,
            list_frames,
            frame_sep,
            file_sep,
            frame_sep_null,
            file_sep_null,
            actions,
            files,
        })
    }

    /// Checks if a command-line argument is a getter argument.
    fn is_getter_arg(arg: &str) -> bool {
        arg.len() == 6 && arg.starts_with("--") && (arg[2..]).chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    }

    /// Checks if a command-line argument is a setter argument.
    fn is_setter_arg(arg: &str) -> bool {
        arg.starts_with("--") && arg.ends_with('=') && matches!(&arg[2..(arg.len() - 1)],
            "COMM" | "TALB" | "TBPM" | "TCAT" | "TCMP" | "TCOM" | "TCON" | "TCOP" |
            "TDAT" | "TDEN" | "TDES" | "TDLY" | "TDOR" | "TDRC" | "TDRL" | "TDTG" |
            "TENC" | "TEXT" | "TFLT" | "TGID" | "TIME" | "TIPL" | "TIT1" | "TIT2" |
            "TIT3" | "TKEY" | "TKWD" | "TLAN" | "TLEN" | "TMCL" | "TMED" | "TMOO" |
            "TOAL" | "TOFN" | "TOLY" | "TOPE" | "TORY" | "TOWN" | "TPE1" | "TPE2" |
            "TPE3" | "TPE4" | "TPOS" | "TPRO" | "TPUB" | "TRCK" | "TRDA" | "TRSN" |
            "TRSO" | "TSIZ" | "TSO2" | "TSOA" | "TSOC" | "TSOP" | "TSOT" | "TSRC" |
            "TSSE" | "TSST" | "TXXX" | "TYER" | "USLT" | "WCOM" | "WCOP" | "WFED" |
            "WOAF" | "WOAR" | "WOAS" | "WORS" | "WPAY" | "WPUB" | "WXXX")
    }

    /// Checks if a command-line argument is a delete argument.
    fn is_delete_arg(arg: &str) -> bool {
        arg.len() == 7 && arg.starts_with("--") && arg.ends_with('-')
        && (arg[2..(arg.len() - 1)]).chars() .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    }
}
