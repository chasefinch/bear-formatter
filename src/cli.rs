//! Command-line interface: argument parsing, in-place formatting, and output.

use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::ExitCode;

use anyhow::Context;
use clap::Parser;

use crate::bear::{self, BearDatabase, Note, Selector};
use crate::engine::Formatter;
use crate::rules;

/// A cute little formatter for Bear notes. 🐻
///
/// Formats in place. Pass a Bear database — its notes are rewritten through
/// Bear's CLI — and/or Markdown files or globs, which are rewritten on disk.
#[derive(Parser)]
#[command(name = "bear-format", version, about)]
struct Cli {
    /// Bear databases and/or Markdown files or globs to format in place.
    #[arg(value_name = "PATH")]
    paths: Vec<String>,

    /// Format a Markdown string and print the result; write nothing.
    #[arg(short = 'c', long, value_name = "MARKDOWN", conflicts_with = "paths")]
    code: Option<String>,

    /// Show what would change without writing anything.
    #[arg(long)]
    dry_run: bool,
}

/// Parse arguments, run the formatter, and return a process exit code.
pub fn run() -> ExitCode {
    match execute(&Cli::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("bear-format: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn execute(cli: &Cli) -> anyhow::Result<ExitCode> {
    let formatter = Formatter::new(rules::all());

    if let Some(markdown) = cli.code.as_deref() {
        print!("{}", formatter.format(markdown));
        return Ok(ExitCode::SUCCESS);
    }
    if cli.paths.is_empty() {
        anyhow::bail!("nothing to do — pass a Bear database or Markdown file/glob, or --code");
    }

    let mut report = Report::default();
    for pattern in &cli.paths {
        let matches =
            glob::glob(pattern).with_context(|| format!("bad path or glob: {pattern}"))?;
        let mut matched_any = false;
        for entry in matches {
            matched_any = true;
            let path = entry?;
            if is_sqlite(&path)? {
                format_database(&path, &formatter, cli.dry_run, &mut report)?;
            } else {
                format_file(&path, &formatter, cli.dry_run, &mut report)?;
            }
        }
        if !matched_any {
            eprintln!("bear-format: no files matched {pattern}");
        }
    }

    report.print(cli.dry_run);
    Ok(report.exit_code())
}

/// Running totals across everything formatted this run.
#[derive(Default)]
struct Report {
    files_changed: usize,
    files_total: usize,
    notes_changed: usize,
    notes_total: usize,
    locked: usize,
    failures: Vec<String>,
}

impl Report {
    fn print(&self, dry_run: bool) {
        let verb = if dry_run { "would be" } else { "were" };
        if self.files_total > 0 {
            println!(
                "{} of {} file(s) {verb} reformatted.",
                self.files_changed, self.files_total
            );
        }
        if self.notes_total > 0 {
            println!(
                "{} of {} note(s) {verb} reformatted.",
                self.notes_changed, self.notes_total
            );
        }
        if self.files_total == 0 && self.notes_total == 0 {
            println!("Nothing to format.");
        }
        if self.locked > 0 {
            println!(
                "Skipped {} locked note(s) — Bear can't read encrypted content.",
                self.locked
            );
        }
        for failure in &self.failures {
            eprintln!("🐾 \x1b[91m{failure}\x1b[0m");
        }
    }

    fn exit_code(&self) -> ExitCode {
        if self.failures.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        }
    }
}

/// Format one Markdown file, rewriting it in place unless `dry_run`.
fn format_file(
    path: &Path,
    formatter: &Formatter,
    dry_run: bool,
    report: &mut Report,
) -> anyhow::Result<()> {
    let original =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    report.files_total += 1;
    let formatted = formatter.format(&original);
    if formatted == original {
        return Ok(());
    }
    report.files_changed += 1;
    if dry_run {
        println!("{}", path.display());
    } else {
        fs::write(path, formatted).with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

/// Format every note in a Bear database, writing each changed note back through
/// bearcli unless `dry_run`.
fn format_database(
    path: &Path,
    formatter: &Formatter,
    dry_run: bool,
    report: &mut Report,
) -> anyhow::Result<()> {
    let database = BearDatabase::open(path)?;
    let notes = database.select(&Selector::All)?;
    report.locked += database.locked_note_count().unwrap_or(0);
    let bearcli = if dry_run {
        None
    } else {
        Some(bear::bearcli_path().context("bearcli not found — is Bear installed?")?)
    };

    for note in &notes {
        report.notes_total += 1;
        let formatted = formatter.format(&note.text);
        if formatted == note.text {
            continue;
        }
        report.notes_changed += 1;
        match &bearcli {
            None => println!("{}  \x1b[2m{}\x1b[0m", label(note), note.identifier),
            Some(cli) => {
                if let Err(error) = bear::overwrite_note(cli, &note.identifier, &formatted) {
                    report.failures.push(format!("{}: {error}", label(note)));
                }
            }
        }
    }
    Ok(())
}

fn label(note: &Note) -> &str {
    if note.title.is_empty() {
        "(untitled)"
    } else {
        note.title.as_str()
    }
}

/// Whether `path` is a SQLite database (by its magic header), not Markdown.
fn is_sqlite(path: &Path) -> anyhow::Result<bool> {
    let mut file = fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut header = [0u8; 16];
    match file.read_exact(&mut header) {
        Ok(()) => Ok(&header == b"SQLite format 3\0"),
        Err(_) => Ok(false),
    }
}
