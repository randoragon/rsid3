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
        println!("\
.-------------------------------------------------------------------------------------------------------------.
|       ID3 standard        | Name |                          Description                         | Writeable |
|-------------------------------------------------------------------------------------------------------------|
|          ID3v2.3, ID3v2.4 | COMM | User comment (DESC, LANG, TEXT)                              |    yes    |
|          ID3v2.3, ID3v2.4 | TALB | Album                                                        |    yes    |
|          ID3v2.3, ID3v2.4 | TBPM | Beats per minute                                             |    yes    |
|          ID3v2.3, ID3v2.4 | TCOM | Composer                                                     |    yes    |
|          ID3v2.3, ID3v2.4 | TCON | Content type (genre)                                         |    yes    |
|          ID3v2.3, ID3v2.4 | TCOP | Copyright                                                    |    yes    |
|          ID3v2.3          | TDAT | Date of recording (DDMM)                                     |    yes    |
|                   ID3v2.4 | TDEN | Encoding time (YYYY-MM-DDTHH:MM:SS)                          |    yes    |
|          ID3v2.3, ID3v2.4 | TDLY | Playlist delay (ms)                                          |    yes    |
|                   ID3v2.4 | TDOR | Original release time (YYYY-MM-DDTHH:MM:SS)                  |    yes    |
|                   ID3v2.4 | TDRC | Recording time (YYYY-MM-DDTHH:MM:SS)                         |    yes    |
|                   ID3v2.4 | TDRL | Release time (YYYY-MM-DDTHH:MM:SS)                           |    yes    |
|                   ID3v2.4 | TDTG | Tagging time (YYYY-MM-DDTHH:MM:SS)                           |    yes    |
|          ID3v2.3, ID3v2.4 | TENC | Encoder                                                      |    yes    |
|          ID3v2.3, ID3v2.4 | TEXT | Lyricist                                                     |    yes    |
|          ID3v2.3, ID3v2.4 | TFLT | File type                                                    |    yes    |
|          ID3v2.3          | TIME | Time of recording (HHMM)                                     |    yes    |
|                   ID3v2.4 | TIPL | Involved people list                                         |    yes    |
|          ID3v2.3, ID3v2.4 | TIT1 | Content group description                                    |    yes    |
|          ID3v2.3, ID3v2.4 | TIT2 | Title                                                        |    yes    |
|          ID3v2.3, ID3v2.4 | TIT3 | Subtitle/description refinement                              |    yes    |
|          ID3v2.3, ID3v2.4 | TKEY | Starting key                                                 |    yes    |
|          ID3v2.3, ID3v2.4 | TLAN | Audio languages                                              |    yes    |
|          ID3v2.3, ID3v2.4 | TLEN | Audio length (ms)                                            |    yes    |
|                   ID3v2.4 | TMCL | Musicians credits list                                       |    yes    |
|          ID3v2.3, ID3v2.4 | TMED | Source media type                                            |    yes    |
|                   ID3v2.4 | TMOO | Mood                                                         |    yes    |
|          ID3v2.3, ID3v2.4 | TOAL | Original album/movie/show title                              |    yes    |
|          ID3v2.3, ID3v2.4 | TOFN | Original filename                                            |    yes    |
|          ID3v2.3, ID3v2.4 | TOLY | Original lyricist                                            |    yes    |
|          ID3v2.3, ID3v2.4 | TOPE | Original artist/performer                                    |    yes    |
|          ID3v2.3          | TORY | Original release year                                        |    yes    |
|          ID3v2.3, ID3v2.4 | TOWN | Owner/Licensee                                               |    yes    |
|          ID3v2.3, ID3v2.4 | TPE1 | Lead artist/performer/soloist/group                          |    yes    |
|          ID3v2.3, ID3v2.4 | TPE2 | Band/Orchestra/Accompaniment                                 |    yes    |
|          ID3v2.3, ID3v2.4 | TPE3 | Conductor                                                    |    yes    |
|          ID3v2.3, ID3v2.4 | TPE4 | Interpreter/Remixer/Modifier                                 |    yes    |
|          ID3v2.3, ID3v2.4 | TPOS | Part of set                                                  |    yes    |
|                   ID3v2.4 | TPRO | Produced                                                     |    yes    |
|          ID3v2.3, ID3v2.4 | TPUB | Publisher                                                    |    yes    |
|          ID3v2.3, ID3v2.4 | TRCK | Track number                                                 |    yes    |
|          ID3v2.3          | TRDA | Recording dates                                              |    yes    |
|          ID3v2.3, ID3v2.4 | TRSN | Internet radio station name                                  |    yes    |
|          ID3v2.3, ID3v2.4 | TRSO | Internet radio station owner                                 |    yes    |
|          ID3v2.3          | TSIZ | Size of audio data (bytes)                                   |    yes    |
|                   ID3v2.4 | TSOA | Album sort order key                                         |    yes    |
|                   ID3v2.4 | TSOP | Performer sort order key                                     |    yes    |
|                   ID3v2.4 | TSOT | Title sort order key                                         |    yes    |
|          ID3v2.3, ID3v2.4 | TSRC | International Standard Recording Code (ISRC)                 |    yes    |
|          ID3v2.3, ID3v2.4 | TSSE | Encoder settings                                             |    yes    |
|                   ID3v2.4 | TSST | Set subtitle                                                 |    yes    |
|          ID3v2.3, ID3v2.4 | TXXX | User-defined text data (DESC, TEXT)                          |    yes    |
|          ID3v2.3          | TYER | Year of recording                                            |    yes    |
|          ID3v2.3, ID3v2.4 | USLT | Unsynchronised lyrics/transcription (DESC, LANG, TEXT)       |    yes    |
|          ID3v2.3, ID3v2.4 | WCOM | Commercial information                                       |    yes    |
|          ID3v2.3, ID3v2.4 | WCOP | Copyright information                                        |    yes    |
|          ID3v2.3, ID3v2.4 | WOAF | Official file information                                    |    yes    |
|          ID3v2.3, ID3v2.4 | WOAR | Official artist/performer information                        |    yes    |
|          ID3v2.3, ID3v2.4 | WOAS | Official source information                                  |    yes    |
|          ID3v2.3, ID3v2.4 | WORS | Official internet radio information                          |    yes    |
|          ID3v2.3, ID3v2.4 | WPAY | Payment information                                          |    yes    |
|          ID3v2.3, ID3v2.4 | WPUB | Official publisher information                               |    yes    |
|          ID3v2.3, ID3v2.4 | WXXX | User-defined URL data (DESC, URL)                            |    yes    |
|          ID3v2.3, ID3v2.4 | AENC | Audio encryption                                             |           |
|          ID3v2.3, ID3v2.4 | APIC | Attached (or linked) picture                                 |           |
|                   ID3v2.4 | ASPI | Audio seek point index                                       |           |
|          ID3v2.3, ID3v2.4 | COMR | Commercial frame                                             |           |
|          ID3v2.3, ID3v2.4 | ENCR | Encryption method registration                               |           |
|          ID3v2.3          | EQUA | Equalization                                                 |           |
|                   ID3v2.4 | EQU2 | Equalization                                                 |           |
|          ID3v2.3, ID3v2.4 | ETCO | Event timing codes                                           |           |
|          ID3v2.3, ID3v2.4 | GEOB | General encapsulated object                                  |           |
|          ID3v2.3, ID3v2.4 | GRID | Group identification registration                            |           |
|          ID3v2.3          | IPLS | Involved people list                                         |           |
|          ID3v2.3, ID3v2.4 | LINK | Linked information                                           |           |
|          ID3v2.3, ID3v2.4 | MCDI | Binary dump of CD's TOC                                      |           |
|          ID3v2.3, ID3v2.4 | MLLT | MPEG location lookup table                                   |           |
|          ID3v2.3, ID3v2.4 | OWNE | Ownership frame                                              |           |
|          ID3v2.3, ID3v2.4 | PCNT | Play counter                                                 |           |
|          ID3v2.3, ID3v2.4 | POPM | Popularimeter                                                |           |
|          ID3v2.3, ID3v2.4 | POSS | Position synchronisation frame                               |           |
|          ID3v2.3, ID3v2.4 | PRIV | Private frame                                                |           |
|          ID3v2.3, ID3v2.4 | RBUF | Recommended buffer size                                      |           |
|          ID3v2.3          | RVAD | Relative volume adjustment                                   |           |
|                   ID3v2.4 | RVA2 | Relative volume adjustment                                   |           |
|          ID3v2.3, ID3v2.4 | RVRB | Reverb                                                       |           |
|                   ID3v2.4 | SEEK | Seek frame                                                   |           |
|                   ID3v2.4 | SIGN | Signature frame                                              |           |
|          ID3v2.3, ID3v2.4 | SYLT | Synchronised lyrics/text                                     |           |
|          ID3v2.3, ID3v2.4 | SYTC | Synchronised tempo codes                                     |           |
|          ID3v2.3, ID3v2.4 | UFID | Unique file identifier                                       |           |
|          ID3v2.3, ID3v2.4 | USER | Terms of use                                                 |           |
| ID3v2.2                   | COM  | Comments (DESC, LANG, TEXT)                                  |    yes    |
| ID3v2.2                   | TAL  | Album/Movie/Show title                                       |    yes    |
| ID3v2.2                   | TBP  | BPM (Beats Per Minute)                                       |    yes    |
| ID3v2.2                   | TCM  | Composer                                                     |    yes    |
| ID3v2.2                   | TCO  | Content type                                                 |    yes    |
| ID3v2.2                   | TCR  | Copyright message                                            |    yes    |
| ID3v2.2                   | TDA  | Date                                                         |    yes    |
| ID3v2.2                   | TDY  | Playlist delay                                               |    yes    |
| ID3v2.2                   | TEN  | Encoded by                                                   |    yes    |
| ID3v2.2                   | TFT  | File type                                                    |    yes    |
| ID3v2.2                   | TIM  | Time                                                         |    yes    |
| ID3v2.2                   | TKE  | Initial key                                                  |    yes    |
| ID3v2.2                   | TLA  | Language(s)                                                  |    yes    |
| ID3v2.2                   | TLE  | Length                                                       |    yes    |
| ID3v2.2                   | TMT  | Media type                                                   |    yes    |
| ID3v2.2                   | TOA  | Original artist(s)/performer(s)                              |    yes    |
| ID3v2.2                   | TOF  | Original filename                                            |    yes    |
| ID3v2.2                   | TOL  | Original Lyricist(s)/text writer(s)                          |    yes    |
| ID3v2.2                   | TOR  | Original release year                                        |    yes    |
| ID3v2.2                   | TOT  | Original album/Movie/Show title                              |    yes    |
| ID3v2.2                   | TP1  | Lead artist(s)/Lead performer(s)/Soloist(s)/Performing group |    yes    |
| ID3v2.2                   | TP2  | Band/Orchestra/Accompaniment                                 |    yes    |
| ID3v2.2                   | TP3  | Conductor/Performer refinement                               |    yes    |
| ID3v2.2                   | TP4  | Interpreted, remixed, or otherwise modified by               |    yes    |
| ID3v2.2                   | TPA  | Part of a set                                                |    yes    |
| ID3v2.2                   | TPB  | Publisher                                                    |    yes    |
| ID3v2.2                   | TRC  | ISRC (International Standard Recording Code)                 |    yes    |
| ID3v2.2                   | TRD  | Recording dates                                              |    yes    |
| ID3v2.2                   | TRK  | Drack number/Position in set                                 |    yes    |
| ID3v2.2                   | TSI  | Size                                                         |    yes    |
| ID3v2.2                   | TSS  | Software/hardware and settings used for encoding             |    yes    |
| ID3v2.2                   | TT1  | Content group description                                    |    yes    |
| ID3v2.2                   | TT2  | Title/Songname/Content description                           |    yes    |
| ID3v2.2                   | TT3  | Subtitle/Description refinement                              |    yes    |
| ID3v2.2                   | TXT  | Lyricist/text writer                                         |    yes    |
| ID3v2.2                   | TXX  | User defined text information frame (DESC, TEXT)             |    yes    |
| ID3v2.2                   | TYE  | Year                                                         |    yes    |
| ID3v2.2                   | ULT  | Unsychronized lyric/text transcription (DESC, LANG, TEXT)    |    yes    |
| ID3v2.2                   | WAF  | Official audio file webpage                                  |    yes    |
| ID3v2.2                   | WAR  | Official artist/performer webpage                            |    yes    |
| ID3v2.2                   | WAS  | Official audio source webpage                                |    yes    |
| ID3v2.2                   | WCM  | Commercial information                                       |    yes    |
| ID3v2.2                   | WCP  | Copyright/Legal information                                  |    yes    |
| ID3v2.2                   | WPB  | Publishers official webpage                                  |    yes    |
| ID3v2.2                   | WXX  | User defined URL link frame (DESC, URL)                      |    yes    |
| ID3v2.2                   | BUF  | Recommended buffer size                                      |           |
| ID3v2.2                   | CNT  | Play counter                                                 |           |
| ID3v2.2                   | CRA  | Audio encryption                                             |           |
| ID3v2.2                   | CRM  | Encrypted meta frame                                         |           |
| ID3v2.2                   | EQU  | Equalization                                                 |           |
| ID3v2.2                   | ETC  | Event timing codes                                           |           |
| ID3v2.2                   | GEO  | General encapsulated object                                  |           |
| ID3v2.2                   | IPL  | Involved people list                                         |           |
| ID3v2.2                   | LNK  | Linked information                                           |           |
| ID3v2.2                   | MCI  | Music CD Identifier                                          |           |
| ID3v2.2                   | MLL  | MPEG location lookup table                                   |           |
| ID3v2.2                   | PIC  | Attached picture                                             |           |
| ID3v2.2                   | POP  | Popularimeter                                                |           |
| ID3v2.2                   | REV  | Reverb                                                       |           |
| ID3v2.2                   | RVA  | Relative volume adjustment                                   |           |
| ID3v2.2                   | SLT  | Synchronized lyric/text                                      |           |
| ID3v2.2                   | STC  | Synced tempo codes                                           |           |
| ID3v2.2                   | UFI  | Unique file identifier                                       |           |
`-------------------------------------------------------------------------------------------------------------`

                                ID3v1                                 ID3v2.3
                        https://id3.org/ID3v1                https://id3.org/id3v2.3.0

                               ID3v2.2                                ID3v2.4
                       https://id3.org/id3v2-00          https://id3.org/id3v2.4.0-frames\n");
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
