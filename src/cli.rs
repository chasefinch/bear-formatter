//! Command-line interface: argument parsing and output.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{ArgGroup, Parser};

use crate::bear::{BearDatabase, Note, Selector};
use crate::config::Config;
use crate::engine::Formatter;
use crate::rules;

/// A cute little formatter for Bear notes. 🐻
#[derive(Parser)]
#[command(name = "bear-formatter", version, about)]
#[command(group(
    ArgGroup::new("target")
        .required(true)
        .multiple(false)
        .args(["note", "tag", "all", "code"])
))]
struct Cli {
    /// Format a single note by its Bear unique identifier.
    #[arg(short = 'n', long, value_name = "UUID")]
    note: Option<String>,

    /// Format every note under a tag, including nested tags.
    #[arg(short = 't', long, value_name = "TAG")]
    tag: Option<String>,

    /// Format the whole database.
    #[arg(short = 'a', long)]
    all: bool,

    /// Format a Markdown string directly and print the result. No database access.
    #[arg(short = 'c', long, value_name = "MARKDOWN")]
    code: Option<String>,

    /// Path to the Bear database (defaults to Bear's group container).
    #[arg(long, value_name = "PATH")]
    database: Option<PathBuf>,
}

/// Parse arguments, run the formatter, and return a process exit code.
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    match execute(&cli) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("bear-formatter: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn execute(cli: &Cli) -> anyhow::Result<ExitCode> {
    let config = Config::discover(&std::env::current_dir()?)?;
    let formatter = Formatter::new(rules::all());

    // The live path today: format a string straight to stdout.
    if let Some(markdown) = cli.code.as_deref() {
        print!("{}", formatter.format(markdown));
        return Ok(ExitCode::SUCCESS);
    }

    let selector = selector_from(cli).expect("clap guarantees exactly one target");
    let path = resolve_database_path(cli, &config)?;
    let notes = BearDatabase::open(&path)?.select(&selector)?;
    report_notes(&formatter, &notes);
    Ok(ExitCode::SUCCESS)
}

fn selector_from(cli: &Cli) -> Option<Selector> {
    if let Some(identifier) = &cli.note {
        return Some(Selector::Note(identifier.clone()));
    }
    if let Some(tag) = &cli.tag {
        return Some(Selector::Tag(tag.clone()));
    }
    if cli.all {
        return Some(Selector::All);
    }
    None
}

fn resolve_database_path(cli: &Cli, config: &Config) -> anyhow::Result<PathBuf> {
    if let Some(path) = &cli.database {
        return Ok(path.clone());
    }
    if let Some(path) = &config.database {
        return Ok(path.clone());
    }
    BearDatabase::default_path()
        .ok_or_else(|| anyhow::anyhow!("could not locate the Bear database; pass --database"))
}

/// Format each note and report which ones would change.
///
/// This is temporary: until write-back through Bear's CLI is wired, database
/// targets read and preview rather than mutate. Once write-back lands, this
/// becomes the actual reformat.
fn report_notes(formatter: &Formatter, notes: &[Note]) {
    if notes.is_empty() {
        println!("No matching notes.");
        return;
    }

    let mut changed = 0;
    for note in notes {
        if formatter.format(&note.text) != note.text {
            changed += 1;
            let title = if note.title.is_empty() {
                "(untitled)"
            } else {
                note.title.as_str()
            };
            println!("\x1b[1m{title}\x1b[0m \x1b[2m{}\x1b[0m", note.identifier);
        }
    }

    if changed == 0 {
        println!("All {} note(s) are already tidy 🐻", notes.len());
    } else {
        println!();
        println!("{changed} of {} note(s) would be reformatted.", notes.len());
        println!("\x1b[2mWrite-back via bearcli isn't enabled yet.\x1b[0m");
    }
}
