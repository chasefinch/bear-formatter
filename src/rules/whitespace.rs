//! **whitespace** — collapse runs of spaces, drop spaces before punctuation,
//! normalize trailing whitespace (keeping two-space hard breaks), and empty
//! whitespace-only lines. Leading indentation and code are left untouched.

use crate::engine::ignore::IgnoreRanges;
use crate::engine::Rule;

pub struct Whitespace;

/// Punctuation that hugs the word before it (no space in between).
const TIGHTENS_LEFT: &[char] = &[',', '.', ';', ':', '!', '?'];

impl Rule for Whitespace {
    fn name(&self) -> &'static str {
        "whitespace"
    }

    fn apply(&self, text: &str, ignore: &IgnoreRanges) -> String {
        let mut out = String::with_capacity(text.len());
        let mut start = 0;
        for piece in text.split_inclusive('\n') {
            let has_newline = piece.ends_with('\n');
            let content = piece.strip_suffix('\n').unwrap_or(piece);
            if ignore.contains(start) {
                out.push_str(content);
            } else {
                out.push_str(&clean_line(content, start, ignore));
            }
            if has_newline {
                out.push('\n');
            }
            start += piece.len();
        }
        out
    }
}

fn clean_line(content: &str, line_start: usize, ignore: &IgnoreRanges) -> String {
    let indent_len = content.len() - content.trim_start_matches([' ', '\t']).len();
    let (indent, body) = content.split_at(indent_len);
    if body.is_empty() {
        return String::new();
    }

    let mut out = String::with_capacity(content.len());
    out.push_str(indent);

    let body_start = line_start + indent_len;
    let mut pending_spaces = 0;
    for (local, character) in body.char_indices() {
        let in_code = ignore.contains(body_start + local);
        if character == ' ' && !in_code {
            pending_spaces += 1;
            continue;
        }
        if pending_spaces > 0 {
            let hugs_left = !in_code && TIGHTENS_LEFT.contains(&character);
            if !hugs_left {
                out.push(' ');
            }
            pending_spaces = 0;
        }
        out.push(character);
    }
    // A two-space (or longer) trailing run is a hard break; keep exactly two.
    if pending_spaces >= 2 {
        out.push_str("  ");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::Whitespace;
    use crate::engine::ignore::IgnoreRanges;
    use crate::engine::Rule;

    fn apply(text: &str) -> String {
        Whitespace.apply(text, &IgnoreRanges::compute(text))
    }

    #[test]
    fn collapses_internal_spaces() {
        assert_eq!(apply("a    b"), "a b");
    }

    #[test]
    fn drops_space_before_punctuation() {
        assert_eq!(apply("hi , there !"), "hi, there!");
    }

    #[test]
    fn preserves_leading_indent() {
        // Two spaces stays a list indent; four would be an indented code block.
        assert_eq!(apply("  - a  b"), "  - a b");
    }

    #[test]
    fn keeps_hard_break_but_strips_single_trailing() {
        assert_eq!(apply("line  \nnext "), "line  \nnext");
    }

    #[test]
    fn empties_whitespace_only_line() {
        assert_eq!(apply("a\n   \nb"), "a\n\nb");
    }

    #[test]
    fn is_idempotent() {
        let once = apply("x  ,  y   z !  ");
        assert_eq!(apply(&once), once);
    }
}
