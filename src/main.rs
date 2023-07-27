use std::fmt;
use std::io;
use std::path::Path;
use std::process::ExitCode;

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

#[cfg(feature = "clap")]
fn parse_args() -> Result<Vec<String>, ExitCode> {
    use clap::{Arg, ArgAction};

    Ok(clap::command!()
        .arg(
            Arg::new("files")
                .required(true)
                .action(ArgAction::Append)
                .value_name("FILE")
                .help("File(s) to dump"),
        )
        .get_matches()
        .get_many("files")
        .expect("no clap files argument")
        .map(String::clone)
        .collect())
}

#[cfg(not(feature = "clap"))]
fn parse_args() -> Result<Vec<String>, ExitCode> {
    static HELP: &str = "\
Usage: tabby FILE [FILE...]
Display the contents of multiple one-line files.

Arguments:
  FILE          File(s) to dump

Options:
  -h --help     Show this help text
";

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprint!("Error: missing argument\n{HELP}");
        return Err(2.into());
    }

    for arg in args.iter() {
        match &**arg {
            "-h" | "--help" => {
                print!("{HELP}");
                return Err(ExitCode::SUCCESS);
            }
            arg if arg.starts_with('-') => {
                eprint!("Error: invalid argument: '{arg}'\n{HELP}");
                return Err(2.into());
            }
            _ => (),
        }
    }

    Ok(args)
}

fn run() -> Result<(), ExitCode> {
    let files: Vec<(String, Text)> = parse_args()?
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
        Err(ExitCode::FAILURE)
    } else {
        Ok(())
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => code,
    }
}
