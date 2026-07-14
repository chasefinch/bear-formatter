//! **final-newline** — a note ends with exactly one trailing newline.
//!
//! The demonstrator rule: small, unambiguous, and total. It proves the engine
//! end-to-end (read → transform) before the real catalog lands.

use crate::engine::ignore::IgnoreRanges;
use crate::engine::Rule;

/// Ensures a single trailing newline.
pub struct FinalNewline;

impl Rule for FinalNewline {
    fn name(&self) -> &'static str {
        "final-newline"
    }

    fn apply(&self, text: &str, _ignore: &IgnoreRanges) -> String {
        if text.is_empty() {
            return String::new();
        }
        // Only the terminal newline run is ours; trailing spaces on the last
        // line belong to a whitespace rule, so leave them be.
        let content = text.trim_end_matches(['\n', '\r']);
        format!("{content}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::FinalNewline;
    use crate::engine::ignore::IgnoreRanges;
    use crate::engine::Rule;

    fn apply(text: &str) -> String {
        FinalNewline.apply(text, &IgnoreRanges::compute(text))
    }

    #[test]
    fn adds_a_missing_newline() {
        assert_eq!(apply("hello"), "hello\n");
    }

    #[test]
    fn collapses_extra_newlines() {
        assert_eq!(apply("hello\n\n\n"), "hello\n");
    }

    #[test]
    fn keeps_a_single_newline() {
        assert_eq!(apply("hello\n"), "hello\n");
    }

    #[test]
    fn leaves_an_empty_note_empty() {
        assert_eq!(apply(""), "");
    }
}
