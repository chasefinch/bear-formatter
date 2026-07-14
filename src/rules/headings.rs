//! **headings** — normalize ATX heading *text*: one space after the `#`s, no
//! leading indentation, no trailing punctuation. Levels are handled separately
//! (see `heading_levels`), and casing is left alone.

use crate::engine::ignore::IgnoreRanges;
use crate::engine::Rule;
use crate::rules::support::heading_level;

pub struct Headings;

const TRAILING_PUNCTUATION: &[char] = &[',', '.', ';', ':', '!', '?'];

impl Rule for Headings {
    fn name(&self) -> &'static str {
        "headings"
    }

    fn apply(&self, text: &str, ignore: &IgnoreRanges) -> String {
        let mut out = String::with_capacity(text.len());
        let mut start = 0;
        for piece in text.split_inclusive('\n') {
            let has_newline = piece.ends_with('\n');
            let content = piece.strip_suffix('\n').unwrap_or(piece);
            let rendered = if ignore.contains(start) {
                content.to_string()
            } else {
                reformat(content)
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

fn reformat(line: &str) -> String {
    let Some(level) = heading_level(line) else {
        return line.to_string();
    };
    let trimmed = line.trim_start_matches([' ', '\t']);
    let body = trimmed[level..].trim();
    let body = body.trim_end_matches(TRAILING_PUNCTUATION).trim_end();
    let hashes = "#".repeat(level);
    if body.is_empty() {
        hashes
    } else {
        format!("{hashes} {body}")
    }
}

#[cfg(test)]
mod tests {
    use super::Headings;
    use crate::engine::ignore::IgnoreRanges;
    use crate::engine::Rule;

    fn apply(text: &str) -> String {
        Headings.apply(text, &IgnoreRanges::compute(text))
    }

    #[test]
    fn collapses_spaces_after_hashes() {
        assert_eq!(apply("##   Foo"), "## Foo");
    }

    #[test]
    fn strips_leading_indent_and_trailing_punctuation() {
        assert_eq!(apply("  # Hello!"), "# Hello");
    }

    #[test]
    fn leaves_tags_alone() {
        assert_eq!(apply("#tag not a heading"), "#tag not a heading");
    }

    #[test]
    fn is_idempotent() {
        let once = apply("###  Title:  ");
        assert_eq!(apply(&once), once);
    }
}
