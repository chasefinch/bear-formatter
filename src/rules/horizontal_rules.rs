//! **horizontal-rules** — divider lines become the canonical `---`.
//!
//! A line holding nothing but dash-family marks, optionally spaced apart —
//! Markdown's `-`/`_`/`*` runs (`-----`, `- - -`, `* * *`) or Unicode dashes
//! pasted as dividers (en/em dashes, the two- and three-em dashes `⸺` `⸻`,
//! figure dash, horizontal bar) — is a horizontal rule and is rewritten as
//! `---`. ASCII marks alone need three or more (a lone `-` is an empty list
//! item, `**` a stray emphasis marker); a single Unicode dash already reads as
//! a full-width rule. Lines inside code, quotes, tables, or any other block
//! construct are not dividers and are left alone.

use crate::engine::ignore::IgnoreRanges;
use crate::engine::Rule;

pub struct HorizontalRules;

/// One of these alone on a line is already a divider.
const STRONG: &[char] = &['‒', '–', '—', '―', '⸺', '⸻'];
/// Markdown's own thematic-break marks; three or more are needed.
const WEAK: &[char] = &['-', '_', '*'];

impl Rule for HorizontalRules {
    fn name(&self) -> &'static str {
        "horizontal-rules"
    }

    fn apply(&self, text: &str, ignore: &IgnoreRanges) -> String {
        let mut out = String::with_capacity(text.len());
        let mut start = 0;
        for piece in text.split_inclusive('\n') {
            let has_newline = piece.ends_with('\n');
            let content = piece.strip_suffix('\n').unwrap_or(piece);
            if !ignore.contains(start) && is_divider(content) {
                out.push_str("---");
            } else {
                out.push_str(content);
            }
            if has_newline {
                out.push('\n');
            }
            start += piece.len();
        }
        out
    }
}

/// Whether the line is only dash-family marks and whitespace, with either a
/// strong (Unicode) dash or at least three marks in total.
fn is_divider(line: &str) -> bool {
    let mut strong = false;
    let mut marks = 0;
    for character in line.chars() {
        if character == ' ' || character == '\t' {
            continue;
        }
        if STRONG.contains(&character) {
            strong = true;
        } else if !WEAK.contains(&character) {
            return false;
        }
        marks += 1;
    }
    marks > 0 && (strong || marks >= 3)
}

#[cfg(test)]
mod tests {
    use super::HorizontalRules;
    use crate::engine::ignore::IgnoreRanges;
    use crate::engine::Rule;

    fn apply(text: &str) -> String {
        HorizontalRules.apply(text, &IgnoreRanges::compute(text))
    }

    #[test]
    fn unicode_dashes_become_rules() {
        for line in ["⸻", "—", "–", "―", "⸺", "‒", "——", "— —"] {
            assert_eq!(apply(line), "---", "for {line:?}");
        }
    }

    #[test]
    fn ascii_runs_of_three_or_more_become_rules() {
        for line in [
            "---", "-----", "___", "****", "- - -", "* * *", "_ _ _", "-\t-\t-",
        ] {
            assert_eq!(apply(line), "---", "for {line:?}");
        }
    }

    #[test]
    fn surrounding_whitespace_is_fine() {
        assert_eq!(apply("  ⸻  "), "---");
        assert_eq!(apply("\t---\t"), "---");
    }

    #[test]
    fn short_ascii_marks_are_not_rules() {
        for line in ["-", "--", "*", "**", "_", "__", "- -"] {
            assert_eq!(apply(line), line, "for {line:?}");
        }
    }

    #[test]
    fn mixed_content_lines_are_left_alone() {
        for line in ["a — b", "—wait", "> ---", "|---|", "3 - 2 - 1", ""] {
            assert_eq!(apply(line), line, "for {line:?}");
        }
    }

    #[test]
    fn leaves_code_alone() {
        let fenced = "```\n---\n⸻\n```";
        assert_eq!(apply(fenced), fenced);
        assert_eq!(apply("`---`"), "`---`");
    }

    #[test]
    fn is_idempotent() {
        let once = apply("a\n⸻\n- - -\nb");
        assert_eq!(apply(&once), once);
    }
}
