//! **tables** — tab-separated lines become Markdown tables, and tables are
//! normalized.
//!
//! A flush-left line that contains a tab and is not already a block construct
//! (heading, list item, blockquote, tag line, footnote, pipe line) converts to
//! a table row — tabs become cell dividers, pipes in cell text are escaped —
//! followed by the minimal separator (`| - | - |`), forming a head-only table.
//! A head-only table separated from a preceding table by nothing but
//! whitespace merges into it as a body row, so `a<tab>b`, blanks, `c<tab>d`
//! reads as a head and a body line. Tables that already have bodies never
//! merge, and pipe blocks without a separator row (address blocks, verse) are
//! left alone.
//!
//! Recognized tables re-emit canonically: cells trimmed to `| cell | cell |`,
//! the separator minimized (alignment colons kept), and ragged rows padded
//! with empty cells to the widest row.

use crate::engine::ignore::IgnoreRanges;
use crate::engine::Rule;
use crate::rules::support::{heading_level, list_item_indent, starts_with_tag};

pub struct Tables;

impl Rule for Tables {
    fn name(&self) -> &'static str {
        "tables"
    }

    fn apply(&self, text: &str, ignore: &IgnoreRanges) -> String {
        let mut lines = Vec::new();
        let mut start = 0;
        for piece in text.split_inclusive('\n') {
            let content = piece.strip_suffix('\n').unwrap_or(piece);
            lines.push((content, start));
            start += piece.len();
        }
        let blocks = merge_tables(parse_blocks(&lines, ignore));
        let joined = render(blocks).join("\n");
        if text.ends_with('\n') {
            format!("{joined}\n")
        } else {
            joined
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Align {
    None,
    Left,
    Right,
    Center,
}

struct Table {
    header: Vec<String>,
    aligns: Vec<Align>,
    body: Vec<Vec<String>>,
}

enum Block<'a> {
    Table(Table),
    /// A run of whitespace-only lines — a candidate merge gap.
    Gap(Vec<&'a str>),
    /// Verbatim lines this rule does not touch.
    Other(Vec<&'a str>),
}

fn parse_blocks<'a>(lines: &[(&'a str, usize)], ignore: &IgnoreRanges) -> Vec<Block<'a>> {
    let mut blocks: Vec<Block<'a>> = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let (content, offset) = lines[index];
        if !ignore.contains(offset) && content.starts_with('|') {
            // A contiguous run of pipe lines: a table when line 2 is a
            // separator row, otherwise a deliberate pipe block left verbatim.
            let run_start = index;
            while index < lines.len() {
                let (line, line_offset) = lines[index];
                if ignore.contains(line_offset) || !line.starts_with('|') {
                    break;
                }
                index += 1;
            }
            let run: Vec<&str> = lines[run_start..index]
                .iter()
                .map(|&(line, _)| line)
                .collect();
            match parse_table(&run) {
                Some(table) => blocks.push(Block::Table(table)),
                None => run.iter().for_each(|line| push_other(&mut blocks, line)),
            }
            continue;
        }

        if ignore.contains(offset) {
            push_other(&mut blocks, content);
        } else if content.trim().is_empty() {
            match blocks.last_mut() {
                Some(Block::Gap(gap)) => gap.push(content),
                _ => blocks.push(Block::Gap(vec![content])),
            }
        } else if let Some(cells) = convert_tab_line(content, offset, ignore) {
            blocks.push(Block::Table(Table {
                aligns: vec![Align::None; cells.len()],
                header: cells,
                body: Vec::new(),
            }));
        } else {
            push_other(&mut blocks, content);
        }
        index += 1;
    }
    blocks
}

fn push_other<'a>(blocks: &mut Vec<Block<'a>>, line: &'a str) {
    match blocks.last_mut() {
        Some(Block::Other(other)) => other.push(line),
        _ => blocks.push(Block::Other(vec![line])),
    }
}

/// If `content` is a convertible tab-separated line, its cells. Convertible
/// means flush left, not an existing block construct, and holding at least one
/// tab outside code spans.
fn convert_tab_line(content: &str, offset: usize, ignore: &IgnoreRanges) -> Option<Vec<String>> {
    if content.starts_with([' ', '\t']) || content.starts_with('>') || content.starts_with("[^") {
        return None;
    }
    if heading_level(content).is_some()
        || starts_with_tag(content)
        || list_item_indent(content).is_some()
    {
        return None;
    }
    let line = content.trim_end();
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut split = false;
    for (local, character) in line.char_indices() {
        if character == '\t' && !ignore.contains(offset + local) {
            cells.push(finish_cell(&cell));
            cell.clear();
            split = true;
        } else {
            cell.push(character);
        }
    }
    if !split {
        return None;
    }
    cells.push(finish_cell(&cell));
    Some(cells)
}

/// Trim a converted cell and escape unescaped pipes so the cell survives
/// re-parsing as one cell.
fn finish_cell(raw: &str) -> String {
    let trimmed = raw.trim();
    let mut out = String::with_capacity(trimmed.len());
    let mut escaped = false;
    for character in trimmed.chars() {
        if character == '|' && !escaped {
            out.push('\\');
        }
        escaped = character == '\\' && !escaped;
        out.push(character);
    }
    out
}

/// Parse a contiguous pipe-line run as a table: header, separator, body rows.
fn parse_table(run: &[&str]) -> Option<Table> {
    if run.len() < 2 {
        return None;
    }
    let aligns = parse_separator(run[1])?;
    Some(Table {
        header: parse_row(run[0]),
        aligns,
        body: run[2..].iter().map(|line| parse_row(line)).collect(),
    })
}

/// Split a table row into trimmed cells at unescaped pipes.
fn parse_row(line: &str) -> Vec<String> {
    let line = line.trim_end();
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut escaped = false;
    let mut closed = false;
    for character in line.chars() {
        if character == '|' && !escaped {
            cells.push(std::mem::take(&mut cell));
            closed = true;
        } else {
            cell.push(character);
            closed = false;
        }
        escaped = character == '\\' && !escaped;
    }
    if !closed {
        // Text after the last pipe is a final cell (no closing pipe).
        cells.push(cell);
    }
    if line.starts_with('|') && !cells.is_empty() {
        // The empty segment before the leading pipe.
        cells.remove(0);
    }
    cells.iter().map(|cell| cell.trim().to_string()).collect()
}

/// Parse a separator row (`| - | :-: |` …) into per-column alignments.
fn parse_separator(line: &str) -> Option<Vec<Align>> {
    let cells = parse_row(line);
    if cells.is_empty() {
        return None;
    }
    let mut aligns = Vec::with_capacity(cells.len());
    for cell in cells {
        let left = cell.starts_with(':');
        let right = cell.len() > 1 && cell.ends_with(':');
        let dashes = cell.trim_start_matches(':').trim_end_matches(':');
        if dashes.is_empty() || !dashes.chars().all(|character| character == '-') {
            return None;
        }
        aligns.push(match (left, right) {
            (false, false) => Align::None,
            (true, false) => Align::Left,
            (false, true) => Align::Right,
            (true, true) => Align::Center,
        });
    }
    Some(aligns)
}

/// Fold head-only tables into the table before them across whitespace-only
/// gaps: the head becomes a body row. Tables with bodies are never absorbed.
fn merge_tables(blocks: Vec<Block>) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::new();
    for block in blocks {
        let Block::Table(table) = block else {
            out.push(block);
            continue;
        };
        if table.body.is_empty() {
            let gap_after_table = matches!(out.last(), Some(Block::Gap(_)))
                && matches!(
                    out.len().checked_sub(2).and_then(|index| out.get(index)),
                    Some(Block::Table(_))
                );
            if gap_after_table {
                out.pop();
            }
            if let Some(Block::Table(previous)) = out.last_mut() {
                previous.body.push(table.header);
                continue;
            }
        }
        out.push(Block::Table(table));
    }
    out
}

fn render(blocks: Vec<Block>) -> Vec<String> {
    let mut lines = Vec::new();
    let mut previous_was_table = false;
    for block in blocks {
        match block {
            Block::Table(table) => {
                // Unmerged adjacent tables need a blank line to stay distinct.
                if previous_was_table {
                    lines.push(String::new());
                }
                render_table(&table, &mut lines);
                previous_was_table = true;
            }
            Block::Gap(verbatim) | Block::Other(verbatim) => {
                lines.extend(verbatim.iter().map(|line| (*line).to_string()));
                previous_was_table = false;
            }
        }
    }
    lines
}

fn render_table(table: &Table, lines: &mut Vec<String>) {
    let width = table
        .header
        .len()
        .max(table.aligns.len())
        .max(table.body.iter().map(Vec::len).max().unwrap_or(0))
        .max(1);
    lines.push(render_row(&table.header, width));
    lines.push(render_separator(&table.aligns, width));
    for row in &table.body {
        lines.push(render_row(row, width));
    }
}

/// A canonical row, padded with empty cells to `width` columns.
fn render_row(cells: &[String], width: usize) -> String {
    let padded: Vec<&str> = (0..width)
        .map(|index| cells.get(index).map_or("", String::as_str))
        .collect();
    format!("| {} |", padded.join(" | "))
}

fn render_separator(aligns: &[Align], width: usize) -> String {
    let padded: Vec<&str> = (0..width)
        .map(
            |index| match aligns.get(index).copied().unwrap_or(Align::None) {
                Align::None => "-",
                Align::Left => ":-",
                Align::Right => "-:",
                Align::Center => ":-:",
            },
        )
        .collect();
    format!("| {} |", padded.join(" | "))
}

#[cfg(test)]
mod tests {
    use super::Tables;
    use crate::engine::ignore::IgnoreRanges;
    use crate::engine::Rule;

    fn apply(text: &str) -> String {
        Tables.apply(text, &IgnoreRanges::compute(text))
    }

    #[test]
    fn converts_a_tab_line_to_a_head_only_table() {
        assert_eq!(apply("a\tb"), "| a | b |\n| - | - |");
    }

    #[test]
    fn adjacent_tab_lines_become_head_and_body() {
        assert_eq!(
            apply("a\tb\nc\td\ne\tf"),
            "| a | b |\n| - | - |\n| c | d |\n| e | f |"
        );
    }

    #[test]
    fn blank_separated_tab_lines_merge_into_one_table() {
        assert_eq!(
            apply("a\tb\n\n\nc\td\n\ne\tf"),
            "| a | b |\n| - | - |\n| c | d |\n| e | f |"
        );
    }

    #[test]
    fn pads_ragged_rows_to_the_widest() {
        assert_eq!(
            apply("a\tb\tc\nd\te"),
            "| a | b | c |\n| - | - | - |\n| d | e |  |"
        );
    }

    #[test]
    fn consecutive_tabs_make_empty_cells() {
        assert_eq!(apply("a\t\tb"), "| a |  | b |\n| - | - | - |");
    }

    #[test]
    fn escapes_pipes_in_converted_cells() {
        assert_eq!(apply("a|b\tc"), "| a\\|b | c |\n| - | - |");
    }

    #[test]
    fn head_only_line_extends_a_preceding_bodied_table() {
        assert_eq!(
            apply("| a | b |\n| - | - |\n| 1 | 2 |\n\nx\ty"),
            "| a | b |\n| - | - |\n| 1 | 2 |\n| x | y |"
        );
    }

    #[test]
    fn bodied_tables_do_not_merge() {
        let two = "| a | b |\n| - | - |\n| 1 | 2 |\n\n| x | y |\n| - | - |\n| 3 | 4 |";
        assert_eq!(apply(two), two);
    }

    #[test]
    fn bodied_table_after_a_tab_line_stays_separate() {
        assert_eq!(
            apply("a\tb\n| x | y |\n| - | - |\n| 1 | 2 |"),
            "| a | b |\n| - | - |\n\n| x | y |\n| - | - |\n| 1 | 2 |"
        );
    }

    #[test]
    fn a_non_tab_line_is_not_a_row_and_blocks_merging() {
        assert_eq!(
            apply("a\tb\nplain\nc\td"),
            "| a | b |\n| - | - |\nplain\n| c | d |\n| - | - |"
        );
    }

    #[test]
    fn leaves_marked_lines_alone() {
        for line in [
            "\ta\tb",
            "# a\tb",
            "#tag\tb",
            "- a\tb",
            "1. a\tb",
            ".\ta\tb",
            "> a\tb",
            "[^1]: a\tb",
        ] {
            assert_eq!(apply(line), line);
        }
    }

    #[test]
    fn leaves_code_alone() {
        let fenced = "```\na\tb\n\nc\td\n```";
        assert_eq!(apply(fenced), fenced);
        assert_eq!(apply("`a\tb`"), "`a\tb`");
    }

    #[test]
    fn splits_only_at_tabs_outside_inline_code() {
        assert_eq!(apply("x\ty `a\tb`"), "| x | y `a\tb` |\n| - | - |");
    }

    #[test]
    fn canonicalizes_an_existing_table() {
        assert_eq!(
            apply("| a  | b |\n|-----|-----|\n| 1 | 2 |"),
            "| a | b |\n| - | - |\n| 1 | 2 |"
        );
    }

    #[test]
    fn keeps_separator_alignment() {
        assert_eq!(
            apply("| a | b | c |\n|:--|--:|:-:|"),
            "| a | b | c |\n| :- | -: | :-: |"
        );
    }

    #[test]
    fn leaves_pipe_blocks_without_a_separator_alone() {
        let verse = "| John Smith\n| 123 Main St\n| Springfield";
        assert_eq!(apply(verse), verse);
    }

    #[test]
    fn is_idempotent() {
        for input in [
            "a\tb\n\nc\td",
            "a\tb\tc\nd\te",
            "a|b\tc",
            "| a | b |\n| - | - |\n| 1 | 2 |\n\n| x | y |\n| - | - |\n| 3 | 4 |",
            "a\tb\n| x | y |\n| - | - |\n| 1 | 2 |",
            "intro\n\na\tb\nc\td\n\ntail",
        ] {
            let once = apply(input);
            assert_eq!(apply(&once), once, "not idempotent for {input:?}");
        }
    }
}
