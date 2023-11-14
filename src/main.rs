use std::fmt;
use std::io;
use std::path::Path;
use std::process::exit;

#[derive(Debug)]
enum Text {
    /// A single line of text. Will *not* contain a trailing newline
    Oneline(String),
    /// More than one line of text. Will contain a trailing newline
    Multiline(String),
    /// An error reading the file
    Err(io::Error),
}

impl Text {
    /// Process a text file's contents, figuring out single-line
    fn new(mut text: String) -> Text {
        fn count_newlines(s: &str) -> usize {
            s.bytes().filter(|b| *b == b'\n').count()
        }

        let trimmed = text.trim_end_matches('\n');
        let orig_count = count_newlines(&text);
        let trimmed_count = count_newlines(trimmed);

        if orig_count == 0 {
            // no newlines in original string
            Text::Oneline(text)
        } else if trimmed_count == 0 {
            // no newlines after we trim the trailing ones
            text.truncate(trimmed.len());
            Text::Oneline(text)
        } else {
            // inner newlines so this is a multi-line string. Make sure there's a trailing one
            if !text.ends_with('\n') {
                text.push('\n');
            }
            Text::Multiline(text)
        }
    }

    /// Read the contents of a file, and strip the terminating newline if exactly one exists.
    pub fn read(path: impl AsRef<Path>) -> Text {
        match std::fs::read_to_string(path) {
            Ok(text) => Text::new(text),
            Err(err) => Text::Err(err),
        }
    }

    #[inline]
    pub fn is_multiline(&self) -> bool {
        matches!(self, Text::Multiline(_))
    }

    #[inline]
    pub fn is_err(&self) -> bool {
        matches!(self, Text::Err(_))
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Text::Oneline(s) | Text::Multiline(s) => f.pad(s),
            Text::Err(err) => write!(f, "[Error: {err}]"),
        }
    }
}

fn parse_args() -> Vec<String> {
    use clap::{Arg, ArgAction, ArgGroup};

    let mut m = clap::command!()
        .arg(
            Arg::new("all")
                .short('a')
                .long("all")
                .action(ArgAction::SetTrue)
                .help("Dump all files in the current directory"),
        )
        .arg(
            Arg::new("files")
                .action(ArgAction::Append)
                .value_name("FILE")
                .help("File(s) to dump"),
        )
        .group(
            ArgGroup::new("file_opts")
                .args(["all", "files"])
                .required(true),
        )
        .get_matches();

    if m.get_flag("all") {
        std::fs::read_dir(".")
            .expect("unable to read contents of current directory")
            .filter_map(|res| res.ok())
            .filter(|ent| ent.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .map(|ent| {
                ent.file_name()
                    .into_string()
                    .expect("filename is not UTF-8")
            })
            .collect()
    } else {
        m.remove_many("files").unwrap().collect()
    }
}

fn main() {
    let files: Vec<(String, Text)> = parse_args()
        .into_iter()
        .map(|path| {
            let text = Text::read(&path);
            (path, text)
        })
        .collect();

    // print one-liners
    let mut had_err = false;
    let max_pathlen = files
        .iter()
        .map(|(path, _)| path.len())
        .max()
        .expect("empty files list");

    for (path, text) in files.iter().filter(|(_, text)| !text.is_multiline()) {
        println!("{path:>max_pathlen$}: {text}");
        had_err |= text.is_err();
    }

    // print multi-liners
    for (path, text) in files.iter().filter(|(_, text)| text.is_multiline()) {
        print!("\n{path}:\n{text}");
    }

    if had_err {
        exit(1)
    }
}
