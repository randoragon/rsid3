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
    pub list_all_frames: bool,
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
        println!("  -l, --list-frames        List supported ID3v2.3 and ID3v2.4 frames.");
        println!("  -L, --list-all-frames    List all supported frames (including ID3v.2.2).");
        println!("  -d SEP, --frame-sep SEP  Separate printed frames with SEP (default: \\n).");
        println!("  -D SEP, --file-sep SEP   Separate printed files with SEP (default: \\n).");
        println!("  -0d, --frame-sep-null    Separate printed frames with the null byte.");
        println!("  -0D, --file-sep-null     Separate printed files with the null byte.");
        println!();
        println!("  --FRAME                  Print the value of FRAME.");
        println!("  --FRAME DESC             Print the value of FRAME (TXXX, WXXX, TXX, WXX).");
        println!("  --FRAME DESC LANG        Print the value of FRAME (COMM, USLT, COM, ULT).");
        println!("  --FRAME= TEXT            Set the value of FRAME.");
        println!("  --FRAME= DESC TEXT       Set the value of FRAME (TXXX, WXXX, TXX, WXX).");
        println!("  --FRAME= DESC LANG TEXT  Set the value of FRAME (COMM, USLT).");
        println!("  --FRAME-                 Delete FRAME.");
        println!("  --FRAME- DESC            Delete FRAME (TXXX, WXXX, TXX, WXX).");
        println!("  --FRAME- DESC LANG       Delete FRAME (COMM, USLT, COM, ULT).");
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
    pub fn print_frames(all: bool) {
        println!("ID3v2.3 and ID3v2.4 common frames:");
        println!("rw	COMM	User comment (DESC, LANG, TEXT)");
        println!("r-	CHAP	Chapter");
        println!("r-	COMR	Commercial frame");
        println!("r-	CTOC	Table of contents");
        println!("r-	ENCR	Encryption method registration");
        println!("r-	ETCO	Event timing codes");
        println!("r-	GEOB	General encapsulated object");
        println!("r-	GRID	Group identification registration");
        println!("r-	GRP1	iTunes grouping (nonstandard)");
        println!("r-	LINK	Linked information");
        println!("r-	MCDI	Binary dump of CD's TOC");
        println!("r-	MLLT	MPEG location lookup table");
        println!("r-	MVIN	iTunes movement number/count (nonstandard)");
        println!("r-	MVNM	iTunes movement name (nonstandard)");
        println!("r-	OWNE	Ownership frame");
        println!("r-	PCNT	Play counter");
        println!("r-	PCST	iTunes podcast flag (nonstandard)");
        println!("r-	POPM	Popularimeter");
        println!("r-	POSS	Position synchronisation frame");
        println!("r-	PRIV	Private frame");
        println!("r-	RBUF	Recommended buffer size");
        println!("r-	RVRB	Reverb");
        println!("r-	SYLT	Synchronised lyrics/text");
        println!("r-	SYTC	Synchronised tempo codes");
        println!("rw	TALB	Album");
        println!("rw	TBPM	Beats per minute");
        println!("rw	TCAT	iTunes podcast category (nonstandard)");
        println!("rw	TCMP	iTunes compilation flag (nonstandard)");
        println!("rw	TCOM	Composer");
        println!("rw	TCON	Content type (genre)");
        println!("rw	TCOP	Copyright");
        println!("rw	TDES	iTunes podcast description (nonstandard)");
        println!("rw	TDLY	Audio delay (ms)");
        println!("rw	TENC	Encoder");
        println!("rw	TEXT	Lyricist");
        println!("rw	TFLT	File type");
        println!("rw	TGID	iTunes podcast identifier (nonstandard)");
        println!("rw	TIT1	Content group description");
        println!("rw	TIT2	Title");
        println!("rw	TIT3	Subtitle/description refinement");
        println!("rw	TKEY	Starting key");
        println!("rw	TKWD	iTunes podcast keywords (nonstandard)");
        println!("rw	TLAN	Audio languages");
        println!("rw	TLEN	Audio length (ms)");
        println!("rw	TMED	Source media type");
        println!("rw	TOAL	Original album");
        println!("rw	TOFN	Original filename");
        println!("rw	TOLY	Original lyricist");
        println!("rw	TOPE	Original artist/performer");
        println!("rw	TOWN	Owner/Licensee");
        println!("rw	TPE1	Lead artist/performer/soloist/group");
        println!("rw	TPE2	Band/Orchestra/Accompaniment");
        println!("rw	TPE3	Conductor");
        println!("rw	TPE4	Interpreter/Remixer/Modifier");
        println!("rw	TPOS	Part of set");
        println!("rw	TPUB	Publisher");
        println!("rw	TRCK	Track number");
        println!("rw	TRSN	Internet radio station name");
        println!("rw	TRSO	Internet radio station owner");
        println!("rw	TSO2	iTunes album artist sort (nonstandard)");
        println!("rw	TSOC	iTunes composer sort (nonstandard)");
        println!("rw	TSRC	International Standard Recording Code (ISRC)");
        println!("rw	TSSE	Encoder settings");
        println!("rw	TXXX	User-defined text data (DESC, TEXT)");
        println!("r-	UFID	Unique file identifier");
        println!("r-	USER	Terms of use");
        println!("rw	USLT	Unsynchronised lyrics/text transcription (DESC, LANG, TEXT)");
        println!("rw	WCOM	Commercial information");
        println!("rw	WCOP	Copyright information");
        println!("rw	WFED	iTunes podcast feed (nonstandard)");
        println!("rw	WOAF	Official file information");
        println!("rw	WOAR	Official artist/performer information");
        println!("rw	WOAS	Official source information");
        println!("rw	WORS	Official internet radio information");
        println!("rw	WPAY	Payment information");
        println!("rw	WPUB	Official publisher information");
        println!("rw	WXXX	User-defined URL data (DESC, URL)");
        println!();
        println!("ID3v2.4 exclusive frames:");
        println!("r-	ASPI	Audio seek point index");
        println!("r-	EQU2	Equalization 2");
        println!("r-	RVA2	Relative volume adjustment 2");
        println!("r-	SEEK	Seek frame");
        println!("r-	SIGN	Signature frame");
        println!("rw	TDEN	Encoding time");
        println!("rw	TDOR	Original release time");
        println!("rw	TDRC	Recording time");
        println!("rw	TDRL	Release time");
        println!("rw	TDTG	Tagging time");
        println!("rw	TIPL	Involved people list");
        println!("rw	TMCL	Musicians credits list");
        println!("rw	TMOO	Mood");
        println!("rw	TPRO	Produced");
        println!("rw	TSOA	Album sort order key");
        println!("rw	TSOP	Performer sort order key");
        println!("rw	TSOT	Title sort order key");
        println!("rw	TSST	Set subtitle");
        println!();
        println!("ID3v2.3 exclusive frames:");
        println!("r-	EQUA	Equalization");
        println!("r-	IPLS	Involved people list");
        println!("r-	RVAD	Relative volume adjustment");
        println!("rw	TDAT	Date of recording (DDMM)");
        println!("rw	TIME	Time of recording (HHMM)");
        println!("rw	TORY	Original release year");
        println!("rw	TRDA	Recording dates");
        println!("rw	TSIZ	Size of audio data (bytes)");
        println!("rw	TYER	Year of recording (YYYY)");
        if all {
            println!();
            println!("ID3v2.2 exclusive frames:");
            println!("r-	BUF	Recommended buffer size");
            println!("r-	CNT	Play counter");
            println!("rw	COM	Comments (DESC, LANG, TEXT)");
            println!("r-	CRA	Audio encryption");
            println!("r-	CRM	Encrypted meta frame");
            println!("r-	ETC	Event timing codes");
            println!("r-	EQU	Equalization");
            println!("r-	GEO	General encapsulated object");
            println!("r-	IPL	Involved people list");
            println!("r-	LNK	Linked information");
            println!("r-	MCI	Music CD Identifier");
            println!("r-	MLL	MPEG location lookup table");
            println!("r-	PIC	Attached picture");
            println!("r-	POP	Popularimeter");
            println!("r-	REV	Reverb");
            println!("r-	RVA	Relative volume adjustment");
            println!("r-	SLT	Synchronized lyric/text");
            println!("r-	STC	Synced tempo codes");
            println!("rw	TAL	Album/Movie/Show title");
            println!("rw	TBP	BPM (Beats Per Minute)");
            println!("rw	TCM	Composer");
            println!("rw	TCO	Content type");
            println!("rw	TCR	Copyright message");
            println!("rw	TDA	Date (DDMM)");
            println!("rw	TDY	Playlist delay");
            println!("rw	TEN	Encoded by");
            println!("rw	TFT	File type");
            println!("rw	TIM	Time (HHMM)");
            println!("rw	TKE	Initial key");
            println!("rw	TLA	Language(s)");
            println!("rw	TLE	Length");
            println!("rw	TMT	Media type");
            println!("rw	TOA	Original artist(s)/performer(s)");
            println!("rw	TOF	Original filename");
            println!("rw	TOL	Original Lyricist(s)/text writer(s)");
            println!("rw	TOR	Original release year");
            println!("rw	TOT	Original album/Movie/Show title");
            println!("rw	TP1	Lead artist(s)/Lead performer(s)/Soloist(s)/Performing group");
            println!("rw	TP2	Band/Orchestra/Accompaniment");
            println!("rw	TP3	Conductor/Performer refinement");
            println!("rw	TP4	Interpreted, remixed, or otherwise modified by");
            println!("rw	TPA	Part of a set");
            println!("rw	TPB	Publisher");
            println!("rw	TRC	ISRC (International Standard Recording Code)");
            println!("rw	TRD	Recording dates");
            println!("rw	TRK	Track number/Position in set");
            println!("rw	TSI	Size");
            println!("rw	TSS	Software/hardware and settings used for encoding");
            println!("rw	TT1	Content group description");
            println!("rw	TT2	Title/Songname/Content description");
            println!("rw	TT3	Subtitle/Description refinement");
            println!("rw	TXT	Lyricist/text writer");
            println!("rw	TXX	User defined text information frame (DESC, TEXT)");
            println!("rw	TYE	Year (YYYY)");
            println!("r-	UFI	Unique file identifier");
            println!("rw	ULT	Unsychronized lyric/text transcription (DESC, LANG, TEXT)");
            println!("rw	WAF	Official audio file webpage");
            println!("rw	WAR	Official artist/performer webpage");
            println!("rw	WAS	Official audio source webpage");
            println!("rw	WCM	Commercial information");
            println!("rw	WCP	Copyright/Legal information");
            println!("rw	WPB	Publishers official webpage");
            println!("rw	WXX	User defined URL link frame (DESC, URL)");
        }
    }

    /// Construct a Cli object representing passed command-line arguments.
    pub fn parse_args() -> Result<Self> {
        let args: Vec<String> = args().collect();
        let mut help = false;
        let mut version = false;
        let mut list_frames = false;
        let mut list_all_frames = false;
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
                "-l" | "--list-frames" => { list_frames = true; },
                "-L" | "--list-all-frames" => { list_all_frames = true; },
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

                "--COMM" | "--COM" => {
                    if i + 2 >= args.len() {
                        return Err(anyhow!("2 arguments expected after {}", args[i]));
                    }
                    let comment = Comment {
                        description: args[i + 1].clone(),
                        lang: args[i + 2].clone(),
                        text: "".to_string(),
                    };
                    actions.push(Action::Print(Frame::with_content("COMM", Content::Comment(comment))));
                    i += 2;
                }
                "--USLT" | "--ULT" => {
                    if i + 2 >= args.len() {
                        return Err(anyhow!("2 arguments expected after {}", args[i]));
                    }
                    let lyrics = Lyrics {
                        description: args[i + 1].clone(),
                        lang: args[i + 2].clone(),
                        text: "".to_string(),
                    };
                    actions.push(Action::Print(Frame::with_content("USLT", Content::Lyrics(lyrics))));
                    i += 2;
                },

                "--TXXX" | "--TXX" => {
                    if i + 1 >= args.len() {
                        return Err(anyhow!("1 argument expected after {}", args[i]));
                    }
                    let extended_text = ExtendedText {
                        value: "".to_string(),
                        description: args[i + 1].clone(),
                    };
                    actions.push(Action::Print(Frame::with_content("TXXX", Content::ExtendedText(extended_text))));
                    i += 1;
                },
                "--WXXX" | "--WXX" => {
                    if i + 1 >= args.len() {
                        return Err(anyhow!("1 argument expected after {}", args[i]));
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

                "--COMM=" | "--COM=" => {
                    if i + 3 >= args.len() {
                        return Err(anyhow!("3 arguments expected after {}", args[i]));
                    }
                    let comment = Comment {
                        description: args[i + 1].clone(),
                        lang: args[i + 2].clone(),
                        text: args[i + 3].clone(),
                    };
                    actions.push(Action::Set(Frame::with_content("COMM", Content::Comment(comment))));
                    i += 3;
                }
                "--USLT=" | "--ULT=" => {
                    if i + 3 >= args.len() {
                        return Err(anyhow!("3 arguments expected after {}", args[i]));
                    }
                    let lyrics = Lyrics {
                        description: args[i + 1].clone(),
                        lang: args[i + 2].clone(),
                        text: args[i + 3].clone(),
                    };
                    actions.push(Action::Set(Frame::with_content("USLT", Content::Lyrics(lyrics))));
                    i += 3;
                }

                "--TXXX=" | "--TXX=" => {
                    if i + 2 >= args.len() {
                        return Err(anyhow!("2 arguments expected after {}", args[i]));
                    }
                    let extended_text = ExtendedText {
                        description: args[i + 1].clone(),
                        value: args[i + 2].clone(),
                    };
                    actions.push(Action::Set(Frame::with_content("TXXX", Content::ExtendedText(extended_text))));
                    i += 2;
                },
                "--WXXX=" | "--WXX=" => {
                    if i + 2 >= args.len() {
                        return Err(anyhow!("2 arguments expected after {}", args[i]));
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

                "--COMM-" | "--COM-" => {
                    if i + 2 >= args.len() {
                        return Err(anyhow!("2 arguments expected after {}", args[i]));
                    }
                    let comment = Comment {
                        description: args[i + 1].clone(),
                        lang: args[i + 2].clone(),
                        text: "".to_string(),
                    };
                    actions.push(Action::Delete(Frame::with_content("COMM", Content::Comment(comment))));
                    i += 2;
                }
                "--USLT-" | "--ULT-" => {
                    if i + 2 >= args.len() {
                        return Err(anyhow!("2 arguments expected after {}", args[i]));
                    }
                    let lyrics = Lyrics {
                        description: args[i + 1].clone(),
                        lang: args[i + 2].clone(),
                        text: "".to_string(),
                    };
                    actions.push(Action::Delete(Frame::with_content("USLT", Content::Lyrics(lyrics))));
                    i += 2;
                },

                "--TXXX-" | "--TXX-" => {
                    if i + 1 >= args.len() {
                        return Err(anyhow!("1 argument expected after {}", &args[i]));
                    }
                    let extended_text = ExtendedText {
                        value: "".to_string(),
                        description: args[i + 1].clone(),
                    };
                    actions.push(Action::Delete(Frame::with_content("TXXX", Content::ExtendedText(extended_text))));
                    i += 1;
                },
                "--WXXX-" | "--WXX-" => {
                    if i + 1 >= args.len() {
                        return Err(anyhow!("1 argument expected after {}", &args[i]));
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
            list_all_frames,
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
        (arg.len() == 6 || arg.len() == 5) && arg.starts_with("--") && (arg[2..]).chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    }

    /// Checks if a command-line argument is a setter argument.
    fn is_setter_arg(arg: &str) -> bool {
        (arg.len() == 7 || arg.len() == 6) && arg.starts_with("--") && arg.ends_with('=')
        && (matches!(&arg[2..(arg.len() - 1)],
            // ID3v2.3 / ID3v2.4 frames
            "COMM" | "TALB" | "TBPM" | "TCAT" | "TCMP" | "TCOM" | "TCON" | "TCOP" |
            "TDAT" | "TDEN" | "TDES" | "TDLY" | "TDOR" | "TDRC" | "TDRL" | "TDTG" |
            "TENC" | "TEXT" | "TFLT" | "TGID" | "TIME" | "TIPL" | "TIT1" | "TIT2" |
            "TIT3" | "TKEY" | "TKWD" | "TLAN" | "TLEN" | "TMCL" | "TMED" | "TMOO" |
            "TOAL" | "TOFN" | "TOLY" | "TOPE" | "TORY" | "TOWN" | "TPE1" | "TPE2" |
            "TPE3" | "TPE4" | "TPOS" | "TPRO" | "TPUB" | "TRCK" | "TRDA" | "TRSN" |
            "TRSO" | "TSIZ" | "TSO2" | "TSOA" | "TSOC" | "TSOP" | "TSOT" | "TSRC" |
            "TSSE" | "TSST" | "TXXX" | "TYER" | "USLT" | "WCOM" | "WCOP" | "WFED" |
            "WOAF" | "WOAR" | "WOAS" | "WORS" | "WPAY" | "WPUB" | "WXXX" |

            // ID3v2.2 frames
            "TT1" | "TT2" | "TT3" | "TP1" | "TP2" | "TP3" | "TP4" | "TCM" |
            "TXT" | "TLA" | "TCO" | "TAL" | "TPA" | "TRK" | "TRC" | "TYE" |
            "TDA" | "TIM" | "TRD" | "TMT" | "TFT" | "TBP" | "TCR" | "TPB" |
            "TEN" | "TSS" | "TOF" | "TLE" | "TSI" | "TDY" | "TKE" | "TOT" |
            "TOA" | "TOL" | "TOR" | "TXX" | "ULT" | "COM")

        // ID3v2.2 URL frames have names "W00" - "WZZ", excluding "WXX".
        // "WXX" is a user-defined URL link frame, which rsid3 also supports.
        || arg[2..(arg.len() - 1)].chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    )
    }

    /// Checks if a command-line argument is a delete argument.
    fn is_delete_arg(arg: &str) -> bool {
        (arg.len() == 7 || arg.len() == 6) && arg.starts_with("--") && arg.ends_with('-')
        && (arg[2..(arg.len() - 1)]).chars() .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    }
}
