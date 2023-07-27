use std::fmt;
use std::io;
use std::path::Path;
use std::process::ExitCode;

use clap::{Arg, ArgAction};

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

fn main() -> ExitCode {
    let args = clap::command!()
        .arg(
            Arg::new("files")
                .required(true)
                .action(ArgAction::Append)
                .value_name("FILE")
                .help("File(s) to dump"),
        )
        .get_matches();

    let files: Vec<(&str, Text)> = args
        .get_many::<String>("files")
        .expect("no clap files argument")
        .map(|path| (&**path, Text::read(path)))
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
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
