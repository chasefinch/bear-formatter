//! **heading-levels** — normalize heading depth.
//!
//! The biggest heading in the note is promoted so it sits just under the note's
//! title: to H1 when the note opens with a heading, or to H2 otherwise (Bear
//! renders a note's first line as its title, so an implicit H1 sits above). The
//! rest shift with it, and no heading may jump more than one level deeper than
//! the previous one. Multiple H1s, and notes with no headings, are both fine.

use crate::engine::ignore::IgnoreRanges;
use crate::engine::Rule;
use crate::rules::support::heading_level;

pub struct HeadingLevels;

const MAX_LEVEL: usize = 6;

impl Rule for HeadingLevels {
    fn name(&self) -> &'static str {
        "heading-levels"
    }

    fn apply(&self, text: &str, ignore: &IgnoreRanges) -> String {
        let Some(smallest) = smallest_heading(text, ignore) else {
            return text.to_string();
        };
        let target_top = if opens_with_heading(text, ignore) {
            1
        } else {
            2
        };

        let mut out = String::with_capacity(text.len());
        let mut previous: Option<usize> = None;
        let mut start = 0;
        for piece in text.split_inclusive('\n') {
            let has_newline = piece.ends_with('\n');
            let content = piece.strip_suffix('\n').unwrap_or(piece);
            let rendered = match heading_level(content) {
                Some(level) if !ignore.contains(start) => {
                    let promoted = promote(level, smallest, target_top);
                    let clamped = match previous {
                        Some(prev) if promoted > prev + 1 => prev + 1,
                        _ => promoted,
                    };
                    previous = Some(clamped);
                    rewrite_level(content, clamped)
                }
                _ => content.to_string(),
            };
            out.push_str(&rendered);
            if has_newline {
                out.push('\n');
            }
            start += piece.len();
        }
        out
    }
}

/// Shift `level` so the note's smallest heading lands at `target_top`, clamped
/// to a valid heading depth.
fn promote(level: usize, smallest: usize, target_top: usize) -> usize {
    let shifted = level as isize - smallest as isize + target_top as isize;
    (shifted.max(1) as usize).min(MAX_LEVEL)
}

/// The smallest heading level present (the biggest heading), or `None`.
fn smallest_heading(text: &str, ignore: &IgnoreRanges) -> Option<usize> {
    let mut smallest: Option<usize> = None;
    let mut start = 0;
    for piece in text.split_inclusive('\n') {
        let content = piece.strip_suffix('\n').unwrap_or(piece);
        if !ignore.contains(start) {
            if let Some(level) = heading_level(content) {
                smallest = Some(smallest.map_or(level, |current| current.min(level)));
            }
        }
        start += piece.len();
    }
    smallest
}

/// Whether the note's first non-blank line is a heading.
fn opens_with_heading(text: &str, ignore: &IgnoreRanges) -> bool {
    let mut start = 0;
    for piece in text.split_inclusive('\n') {
        let content = piece.strip_suffix('\n').unwrap_or(piece);
        if !content.trim().is_empty() {
            return !ignore.contains(start) && heading_level(content).is_some();
        }
        start += piece.len();
    }
    false
}

/// Rewrite a heading line to `level` `#`s, preserving the text after them.
fn rewrite_level(line: &str, level: usize) -> String {
    let trimmed = line.trim_start_matches([' ', '\t']);
    let hashes = trimmed.bytes().take_while(|&byte| byte == b'#').count();
    format!("{}{}", "#".repeat(level), &trimmed[hashes..])
}

#[cfg(test)]
mod tests {
    use super::HeadingLevels;
    use crate::engine::ignore::IgnoreRanges;
    use crate::engine::Rule;

    fn apply(text: &str) -> String {
        HeadingLevels.apply(text, &IgnoreRanges::compute(text))
    }

    #[test]
    fn promotes_biggest_to_h1_when_opening_with_a_heading() {
        assert_eq!(apply("### A\n#### B"), "# A\n## B");
    }

    #[test]
    fn promotes_biggest_to_h2_when_opening_with_text() {
        assert_eq!(
            apply("Intro.\n### Section\n#### Sub"),
            "Intro.\n## Section\n### Sub"
        );
    }

    #[test]
    fn demotes_a_late_h1_under_the_implicit_title() {
        assert_eq!(
            apply("Title text\n# Big\n## Small"),
            "Title text\n## Big\n### Small"
        );
    }

    #[test]
    fn clamps_deeper_jumps() {
        assert_eq!(apply("# A\n### B"), "# A\n## B");
    }

    #[test]
    fn keeps_multiple_h1s() {
        assert_eq!(apply("# A\n# B"), "# A\n# B");
    }

    #[test]
    fn leaves_headingless_notes_alone() {
        assert_eq!(apply("just text\nmore"), "just text\nmore");
    }

    #[test]
    fn is_idempotent() {
        let once = apply("Intro\n## A\n#### B\n### C");
        assert_eq!(apply(&once), once);
    }
}
