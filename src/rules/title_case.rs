//! **title-case** — the note's title (a first-line heading) is title-cased.
//!
//! Only the first non-blank line, and only when it is a heading, is touched —
//! every other heading keeps its casing. Words that already contain an interior
//! capital (URL, iPhone, McDonald) are left alone, and nothing is ever
//! lowercased except small words (articles, short conjunctions and prepositions)
//! in the middle of the title. First and last words are always capitalized.

use crate::engine::ignore::IgnoreRanges;
use crate::engine::Rule;
use crate::rules::support::heading_level;

pub struct TitleCase;

/// Words kept lowercase in the middle of a title (an AP-ish subset).
const SMALL_WORDS: &[&str] = &[
    "a", "an", "and", "as", "at", "but", "by", "for", "if", "in", "nor", "of", "on", "or", "per",
    "the", "to", "up", "via", "vs",
];

impl Rule for TitleCase {
    fn name(&self) -> &'static str {
        "title-case"
    }

    fn apply(&self, text: &str, ignore: &IgnoreRanges) -> String {
        let mut out = String::with_capacity(text.len());
        let mut handled = false;
        let mut start = 0;
        for piece in text.split_inclusive('\n') {
            let has_newline = piece.ends_with('\n');
            let content = piece.strip_suffix('\n').unwrap_or(piece);
            let rendered = if !handled && !content.trim().is_empty() {
                handled = true;
                match heading_level(content) {
                    Some(level) if !ignore.contains(start) => retitle(content, level),
                    _ => content.to_string(),
                }
            } else {
                content.to_string()
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

fn retitle(line: &str, level: usize) -> String {
    let body = line.trim_start_matches([' ', '\t'])[level..].trim();
    if body.is_empty() {
        return line.to_string();
    }
    let words: Vec<&str> = body.split_whitespace().collect();
    let last = words.len() - 1;
    let cased: Vec<String> = words
        .iter()
        .enumerate()
        .map(|(index, word)| title_word(word, index == 0 || index == last))
        .collect();
    format!("{} {}", "#".repeat(level), cased.join(" "))
}

fn title_word(word: &str, is_edge: bool) -> String {
    if word.chars().skip(1).any(char::is_uppercase) {
        return word.to_string(); // interior capital — leave it (URL, iPhone)
    }
    if !is_edge && SMALL_WORDS.contains(&word.to_lowercase().as_str()) {
        return word.to_lowercase();
    }
    capitalize_first(word)
}

fn capitalize_first(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::TitleCase;
    use crate::engine::ignore::IgnoreRanges;
    use crate::engine::Rule;

    fn apply(text: &str) -> String {
        TitleCase.apply(text, &IgnoreRanges::compute(text))
    }

    #[test]
    fn title_cases_the_first_heading() {
        assert_eq!(apply("# the quick brown fox"), "# The Quick Brown Fox");
    }

    #[test]
    fn keeps_interior_capitals_and_lowercases_small_words() {
        assert_eq!(
            apply("# the URL of an API spec"),
            "# The URL of an API Spec"
        );
    }

    #[test]
    fn leaves_non_first_headings_and_prose_alone() {
        assert_eq!(
            apply("intro\n# a later heading"),
            "intro\n# a later heading"
        );
        assert_eq!(
            apply("# first\n## second heading"),
            "# First\n## second heading"
        );
    }

    #[test]
    fn is_idempotent() {
        let once = apply("# a study of the iPhone URL scheme");
        assert_eq!(apply(&once), once);
    }
}
