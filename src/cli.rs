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
use id3::Content;
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

/// Represents a single "frame argument" with its associated subarguments,
/// e.g. --TIT2, --TT2= val, etc.
#[derive(Debug, Clone)]
pub struct Frame {
    pub id: String,
    pub desc: Option<String>,
    pub lang: Option<String>,
    pub content: Option<String>,
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
        println!("  -d SEP, --frame-sep SEP  Separate printed frames with SEP (default: \"\\n\").");
        println!("  -D SEP, --file-sep SEP   Separate printed files with SEP (default: \"\\n\").");
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
        println!("\
.-------------------------------------------------------------------------------------------------.
| ID3 standard  | Name | Alias |                          Description                   | Support |
|-------------------------------------------------------------------------------------------------|
| 2.2, 2.3, 2.4 | AENC |  CRA  | Audio encryption                                       |    r    |
| 2.2, 2.3, 2.4 | APIC |  PIC  | Attached (or linked) picture                           |    r    |
|           2.4 | ASPI |       | Audio seek point index                                 |    r    |
|      2.3, 2.4 | CHAP |       | Chapter                                                |    r    |
| 2.2, 2.3, 2.4 | COMM |  COM  | User comment (DESC, LANG, TEXT)                        |   r/w   |
|      2.3, 2.4 | COMR |       | Commercial frame                                       |    r    |
| 2.2           | CRM  |       | Encrypted meta frame                                   |    r    |
|      2.3, 2.4 | CTOC |       | Table of contents                                      |    r    |
|      2.3, 2.4 | ENCR |       | Encryption method registration                         |    r    |
| 2.2, 2.3      | EQUA |  EQU  | Equalization                                           |    r    |
|           2.4 | EQU2 |       | Equalization 2                                         |    r    |
| 2.2, 2.3, 2.4 | ETCO |  ETC  | Event timing codes                                     |    r    |
| 2.2, 2.3, 2.4 | GEOB |  GEO  | General encapsulated object                            |    r    |
|      2.3, 2.4 | GRID |       | Group identification registration                      |    r    |
|      2.3, 2.4 | GRP1 |       | iTunes grouping (nonstandard)                          |    r    |
| 2.2, 2.3      | IPLS |  IPL  | Involved people list                                   |    r    |
| 2.2, 2.3, 2.4 | LINK |  LNK  | Linked information                                     |    r    |
| 2.2, 2.3, 2.4 | MCDI |  MCI  | Binary dump of CD's TOC                                |    r    |
| 2.2, 2.3, 2.4 | MLLT |  MLL  | MPEG location lookup table                             |    r    |
|      2.3, 2.4 | MVIN |       | iTunes movement number/count (nonstandard)             |    r    |
|      2.3, 2.4 | MVNM |       | iTunes movement name (nonstandard)                     |    r    |
|      2.3, 2.4 | OWNE |       | Ownership frame                                        |    r    |
| 2.2, 2.3, 2.4 | PCNT |  CNT  | Play counter                                           |    r    |
|      2.3, 2.4 | PCST |       | iTunes podcast flag (nonstandard)                      |    r    |
| 2.2, 2.3, 2.4 | POPM |  POP  | Popularimeter                                          |    r    |
|      2.3, 2.4 | POSS |       | Position synchronisation frame                         |    r    |
|      2.3, 2.4 | PRIV |       | Private frame                                          |    r    |
| 2.2, 2.3, 2.4 | RBUF |  BUF  | Recommended buffer size                                |    r    |
|      2.3      | RVAD |       | Relative volume adjustment                             |    r    |
| 2.2,      2.4 | RVA2 |  RVA  | Relative volume adjustment 2                           |    r    |
| 2.2, 2.3, 2.4 | RVRB |  REV  | Reverb                                                 |    r    |
|           2.4 | SEEK |       | Seek frame                                             |    r    |
|           2.4 | SIGN |       | Signature frame                                        |    r    |
| 2.2, 2.3, 2.4 | SYLT |  SLT  | Synchronised lyrics/text                               |    r    |
| 2.2, 2.3, 2.4 | SYTC |  STC  | Synchronised tempo codes                               |    r    |
| 2.2, 2.3, 2.4 | TALB |  TAL  | Album                                                  |   r/w   |
| 2.2, 2.3, 2.4 | TBPM |  TBP  | Beats per minute                                       |   r/w   |
|      2.3, 2.4 | TCAT |       | iTunes podcast category (nonstandard)                  |   r/w   |
|      2.3, 2.4 | TCMP |       | iTunes compilation flag (nonstandard)                  |   r/w   |
| 2.2, 2.3, 2.4 | TCOM |  TCM  | Composer                                               |   r/w   |
| 2.2, 2.3, 2.4 | TCON |  TCO  | Content type (genre)                                   |   r/w   |
| 2.2, 2.3, 2.4 | TCOP |  TCR  | Copyright                                              |   r/w   |
| 2.2, 2.3      | TDAT |  TDA  | Date of recording (DDMM)                               |   r/w   |
|           2.4 | TDEN |       | Encoding time (YYYY-MM-DDTHH:MM:SS)                    |   r/w   |
|      2.3, 2.4 | TDES |       | iTunes podcast description (nonstandard)               |   r/w   |
| 2.2, 2.3, 2.4 | TDLY |  TDY  | Playlist delay (ms)                                    |   r/w   |
|           2.4 | TDOR |       | Original release time (YYYY-MM-DDTHH:MM:SS)            |   r/w   |
|           2.4 | TDRC |       | Recording time (YYYY-MM-DDTHH:MM:SS)                   |   r/w   |
|           2.4 | TDRL |       | Release time (YYYY-MM-DDTHH:MM:SS)                     |   r/w   |
|           2.4 | TDTG |       | Tagging time (YYYY-MM-DDTHH:MM:SS)                     |   r/w   |
| 2.2, 2.3, 2.4 | TENC |  TEN  | Encoder                                                |   r/w   |
| 2.2, 2.3, 2.4 | TEXT |  TXT  | Lyricist                                               |   r/w   |
| 2.2, 2.3, 2.4 | TFLT |  TFT  | File type                                              |   r/w   |
|      2.3, 2.4 | TGID |       | iTunes podcast identifier (nonstandard)                |   r/w   |
| 2.2, 2.3      | TIME |       | Time of recording (HHMM)                               |   r/w   |
|           2.4 | TIPL |       | Involved people list                                   |    r    |
| 2.2, 2.3, 2.4 | TIT1 |  TT1  | Content group description                              |   r/w   |
| 2.2, 2.3, 2.4 | TIT2 |  TT2  | Title                                                  |   r/w   |
| 2.2, 2.3, 2.4 | TIT3 |  TT3  | Subtitle/description refinement                        |   r/w   |
| 2.2, 2.3, 2.4 | TKEY |  TKE  | Starting key                                           |   r/w   |
|      2.3, 2.4 | TKWD |       | iTunes podcast keywords (nonstandard)                  |   r/w   |
| 2.2, 2.3, 2.4 | TLAN |  TLA  | Audio languages                                        |   r/w   |
| 2.2, 2.3, 2.4 | TLEN |  TLE  | Audio length (ms)                                      |   r/w   |
|           2.4 | TMCL |       | Musicians credits list                                 |    r    |
| 2.2, 2.3, 2.4 | TMED |  TMT  | Source media type                                      |   r/w   |
|           2.4 | TMOO |       | Mood                                                   |   r/w   |
| 2.2, 2.3, 2.4 | TOAL |  TOT  | Original album/movie/show title                        |   r/w   |
| 2.2, 2.3, 2.4 | TOFN |  TOF  | Original filename                                      |   r/w   |
| 2.2, 2.3, 2.4 | TOLY |  TOL  | Original lyricist                                      |   r/w   |
| 2.2, 2.3, 2.4 | TOPE |  TOA  | Original artist/performer                              |   r/w   |
| 2.2, 2.3      | TORY |  TOR  | Original release year                                  |   r/w   |
|      2.3, 2.4 | TOWN |       | Owner/Licensee                                         |   r/w   |
| 2.2, 2.3, 2.4 | TPE1 |  TP1  | Lead artist/performer/soloist/group                    |   r/w   |
| 2.2, 2.3, 2.4 | TPE2 |  TP2  | Band/Orchestra/Accompaniment                           |   r/w   |
| 2.2, 2.3, 2.4 | TPE3 |  TP3  | Conductor                                              |   r/w   |
| 2.2, 2.3, 2.4 | TPE4 |  TP4  | Interpreter/Remixer/Modifier                           |   r/w   |
| 2.2, 2.3, 2.4 | TPOS |  TPA  | Part of set                                            |   r/w   |
|           2.4 | TPRO |       | Produced                                               |   r/w   |
| 2.2, 2.3, 2.4 | TPUB |  TPB  | Publisher                                              |   r/w   |
| 2.2, 2.3, 2.4 | TRCK |  TRK  | Track number                                           |   r/w   |
| 2.2, 2.3      | TRDA |  TRD  | Recording dates                                        |   r/w   |
|      2.3, 2.4 | TRSN |       | Internet radio station name                            |   r/w   |
|      2.3, 2.4 | TRSO |       | Internet radio station owner                           |   r/w   |
| 2.2, 2.3      | TSIZ |  TSI  | Size of audio data (bytes)                             |   r/w   |
|      2.3, 2.4 | TSO2 |       | iTunes album artist sort (nonstandard)                 |   r/w   |
|           2.4 | TSOA |       | Album sort order key                                   |   r/w   |
|      2.3, 2.4 | TSOC |       | iTunes composer sort (nonstandard)                     |   r/w   |
|           2.4 | TSOP |       | Performer sort order key                               |   r/w   |
|           2.4 | TSOT |       | Title sort order key                                   |   r/w   |
| 2.2, 2.3, 2.4 | TSRC |  TRC  | International Standard Recording Code (ISRC)           |   r/w   |
| 2.2, 2.3, 2.4 | TSSE |  TSS  | Encoder settings                                       |   r/w   |
|           2.4 | TSST |       | Set subtitle                                           |   r/w   |
| 2.2, 2.3, 2.4 | TXXX |  TXX  | User-defined text data (DESC, TEXT)                    |   r/w   |
| 2.2, 2.3      | TYER |  TYE  | Year of recording (YYYY)                               |   r/w   |
| 2.2, 2.3, 2.4 | UFID |  UFI  | Unique file identifier                                 |    r    |
|      2.3, 2.4 | USER |       | Terms of use                                           |    r    |
| 2.2, 2.3, 2.4 | USLT |  ULT  | Unsynchronised lyrics/transcription (DESC, LANG, TEXT) |   r/w   |
| 2.2, 2.3, 2.4 | WCOM |  WCM  | Commercial information                                 |   r/w   |
| 2.2, 2.3, 2.4 | WCOP |  WCP  | Copyright information                                  |   r/w   |
|      2.3, 2.4 | WFED |       | iTunes podcast feed (nonstandard)                      |   r/w   |
| 2.2, 2.3, 2.4 | WOAF |  WAF  | Official file information                              |   r/w   |
| 2.2, 2.3, 2.4 | WOAR |  WAR  | Official artist/performer information                  |   r/w   |
| 2.2, 2.3, 2.4 | WOAS |  WAS  | Official source information                            |   r/w   |
|      2.3, 2.4 | WORS |       | Official internet radio information                    |   r/w   |
|      2.3, 2.4 | WPAY |       | Payment information                                    |   r/w   |
| 2.2, 2.3, 2.4 | WPUB |  WPB  | Official publisher information                         |   r/w   |
| 2.2, 2.3, 2.4 | WXXX |  WXX  | User-defined URL data (DESC, URL)                      |   r/w   |
`-------------------------------------------------------------------------------------------------`

                           ID3v1                                 ID3v2.3
                   http://id3.org/ID3v1                  http://id3.org/id3v2.3.0

                          ID3v2.2                                ID3v2.4
                  http://id3.org/id3v2-00            http://id3.org/id3v2.4.0-frames\n");
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

                "--COMM" | "--COM" |
                "--USLT" | "--ULT" => {
                    if i + 2 >= args.len() {
                        return Err(anyhow!("2 arguments expected after {}", args[i]));
                    }
                    let frame = Frame {
                        id: arg[2..].to_string(),
                        desc: Some(args[i + 1].clone()),
                        lang: Some(args[i + 2].clone()),
                        content: None,
                    };
                    actions.push(Action::Print(frame));
                    i += 2;
                },

                "--TXXX" | "--TXX" |
                "--WXXX" | "--WXX" => {
                    if i + 1 >= args.len() {
                        return Err(anyhow!("1 argument expected after {}", args[i]));
                    }
                    let frame = Frame {
                        id: arg[2..].to_string(),
                        desc: Some(args[i + 1].clone()),
                        lang: None,
                        content: None,
                    };
                    actions.push(Action::Print(frame));
                    i += 1;
                },

                // All parameterless getters
                str if Cli::is_getter_arg(str) => {
                    let frame = Frame {
                        id: str[2..].to_string(),
                        desc: None,
                        lang: None,
                        content: None,
                    };
                    actions.push(Action::Print(frame));
                },

                "--COMM=" | "--COM=" |
                "--USLT=" | "--ULT=" => {
                    if i + 3 >= args.len() {
                        return Err(anyhow!("3 arguments expected after {}", args[i]));
                    }
                    let frame = Frame {
                        id: arg[2..(arg.len() - 1)].to_string(),
                        desc: Some(args[i + 1].clone()),
                        lang: Some(args[i + 2].clone()),
                        content: Some(args[i + 3].clone()),
                    };
                    actions.push(Action::Set(frame));
                    i += 3;
                }

                "--TXXX=" | "--TXX=" |
                "--WXXX=" | "--WXX=" => {
                    if i + 2 >= args.len() {
                        return Err(anyhow!("2 arguments expected after {}", args[i]));
                    }
                    let frame = Frame {
                        id: arg[2..(arg.len() - 1)].to_string(),
                        desc: Some(args[i + 1].clone()),
                        lang: None,
                        content: Some(args[i + 2].clone()),
                    };
                    actions.push(Action::Set(frame));
                    i += 2;
                },

                // All parameterless setters
                str if Cli::is_setter_arg(str) => {
                    if i + 1 >= args.len() {
                        return Err(anyhow!("1 argument expected after {str}"));
                    }
                    let frame = Frame {
                        id: str[2..(str.len() - 1)].to_string(),
                        desc: None,
                        lang: None,
                        content: Some(args[i + 1].clone()),
                    };
                    actions.push(Action::Set(frame));
                    i += 1;
                },

                "--COMM-" | "--COM-" |
                "--USLT-" | "--ULT-" => {
                    if i + 2 >= args.len() {
                        return Err(anyhow!("2 arguments expected after {}", args[i]));
                    }
                    let frame = Frame {
                        id: arg[2..(arg.len() - 1)].to_string(),
                        desc: Some(args[i + 1].clone()),
                        lang: Some(args[i + 2].clone()),
                        content: None,
                    };
                    actions.push(Action::Delete(frame));
                    i += 2;
                }

                "--TXXX-" | "--TXX-" |
                "--WXXX-" | "--WXX-" => {
                    if i + 1 >= args.len() {
                        return Err(anyhow!("1 argument expected after {}", args[i]));
                    }
                    let frame = Frame {
                        id: arg[2..(arg.len() - 1)].to_string(),
                        desc: Some(args[i + 1].clone()),
                        lang: None,
                        content: None,
                    };
                    actions.push(Action::Delete(frame));
                    i += 1;
                },

                // All parameterless delete args
                str if Cli::is_delete_arg(str) => {
                    let frame = Frame {
                        id: arg[2..(arg.len() - 1)].to_string(),
                        desc: None,
                        lang: None,
                        content: None,
                    };
                    actions.push(Action::Delete(frame));
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

impl Frame {
    /// Converts a Cli::Frame to an id3::Frame. This is a one-way conversion, because id3::Frame
    /// does not retain information about whether a frame was ID3v2.2 or ID3v2.3+. In particular,
    /// the 3-letter frame ID of ID3v2.2 gets converted to a 4-letter ID of ID3v2.3.
    pub fn to_id3_frame(&self) -> id3::Frame {
        match self.id.as_str() {
            "COMM" | "COM" => {
                let comment = Comment {
                    description: self.desc.clone().unwrap(),
                    lang: self.lang.clone().unwrap(),
                    text: self.content.clone().unwrap_or_default(),
                };
                id3::Frame::with_content(self.id.clone(), Content::Comment(comment))
            },

            "USLT" | "ULT" => {
                let lyrics = Lyrics {
                    description: self.desc.clone().unwrap(),
                    lang: self.lang.clone().unwrap(),
                    text: self.content.clone().unwrap_or_default(),
                };
                id3::Frame::with_content(self.id.clone(), Content::Lyrics(lyrics))
            },

            "TXXX" | "TXX" => {
                let extended_text = ExtendedText {
                    description: self.desc.clone().unwrap(),
                    value: self.content.clone().unwrap_or_default(),
                };
                id3::Frame::with_content(self.id.clone(), Content::ExtendedText(extended_text))
            },

            "WXXX" | "WXX" => {
                let extended_link = ExtendedLink {
                    description: self.desc.clone().unwrap(),
                    link: self.content.clone().unwrap_or_default(),
                };
                id3::Frame::with_content(self.id.clone(), Content::ExtendedLink(extended_link))
            },

            _ => id3::Frame::text(self.id.clone(), self.content.clone().unwrap_or_default()),
        }
    }
}
