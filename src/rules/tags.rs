//! **tags** — Bear tags, tidied.
//!
//! A line made up entirely of tags is metadata: such lines are gathered,
//! deduped, sorted, and moved to the top — right under the first heading, or
//! the very top if there is none — on a single line. A line that mixes tags
//! with prose stays put, but each tag begins a new line carrying the text that
//! follows it. A redundant closing `#` is stripped from any tag without spaces.
//! Tags inside code are ignored.

use std::collections::HashSet;

use crate::engine::ignore::IgnoreRanges;
use crate::engine::Rule;
use crate::rules::support::heading_level;

pub struct Tags;

enum Classified {
    Verbatim { text: String, heading: bool },
    Replaced(Vec<String>),
    Removed,
}

impl Rule for Tags {
    fn name(&self) -> &'static str {
        "tags"
    }

    fn apply(&self, text: &str, ignore: &IgnoreRanges) -> String {
        let mut classified = Vec::new();
        let mut gathered = Vec::new();
        let mut start = 0;
        for piece in text.split_inclusive('\n') {
            let content = piece.strip_suffix('\n').unwrap_or(piece);
            if ignore.contains(start) {
                classified.push(Classified::Verbatim {
                    text: content.to_string(),
                    heading: false,
                });
            } else if let Some(tags) = pure_tag_line(content) {
                gathered.extend(tags);
                classified.push(Classified::Removed);
            } else if is_mixed_tag_line(content) {
                classified.push(Classified::Replaced(split_mixed(content)));
            } else {
                classified.push(Classified::Verbatim {
                    text: content.to_string(),
                    heading: heading_level(content).is_some(),
                });
            }
            start += piece.len();
        }

        let mut lines = Vec::new();
        let mut anchor = None;
        for line in classified {
            match line {
                Classified::Verbatim { text, heading } => {
                    if heading && anchor.is_none() {
                        anchor = Some(lines.len());
                    }
                    lines.push(text);
                }
                Classified::Replaced(replacements) => lines.extend(replacements),
                Classified::Removed => {}
            }
        }

        if !gathered.is_empty() {
            let insert_at = anchor.map_or(0, |index| index + 1);
            lines.insert(insert_at, merge_and_sort(gathered));
        }

        let joined = lines.join("\n");
        if text.ends_with('\n') {
            format!("{joined}\n")
        } else {
            joined
        }
    }
}

/// If every whitespace-separated token is a tag, return them normalized.
fn pure_tag_line(line: &str) -> Option<Vec<String>> {
    let tags: Option<Vec<String>> = line.split_whitespace().map(normalize_tag).collect();
    tags.filter(|found| !found.is_empty())
}

/// Whether `line` starts with a tag but also carries non-tag text.
fn is_mixed_tag_line(line: &str) -> bool {
    let mut tokens = line.split_whitespace();
    match tokens.next() {
        Some(first) if normalize_tag(first).is_some() => {
            tokens.any(|token| normalize_tag(token).is_none())
        }
        _ => false,
    }
}

/// Split a mixed line so each tag starts a new line carrying the text after it,
/// preserving the original indentation.
fn split_mixed(line: &str) -> Vec<String> {
    let indent = &line[..line.len() - line.trim_start().len()];
    let mut lines = Vec::new();
    let mut current = String::new();
    for token in line.split_whitespace() {
        let normalized = normalize_tag(token);
        if normalized.is_some() && !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        if current.is_empty() {
            current.push_str(indent);
        } else {
            current.push(' ');
        }
        current.push_str(&normalized.unwrap_or_else(|| token.to_string()));
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// If `token` is a single tag, return it normalized (a redundant trailing `#`
/// stripped). A tag is `#` then non-space, non-`#` characters, optionally closed
/// by one `#`.
fn normalize_tag(token: &str) -> Option<String> {
    let inner = token.strip_prefix('#')?;
    let core = inner.strip_suffix('#').unwrap_or(inner);
    if core.is_empty() || core.contains('#') {
        return None;
    }
    Some(format!("#{core}"))
}

fn merge_and_sort(tags: Vec<String>) -> String {
    let mut seen = HashSet::new();
    let mut unique = Vec::new();
    for tag in tags {
        if seen.insert(tag.to_lowercase()) {
            unique.push(tag);
        }
    }
    unique.sort_by_key(|tag| tag.to_lowercase());
    unique.join(" ")
}

#[cfg(test)]
mod tests {
    use super::Tags;
    use crate::engine::ignore::IgnoreRanges;
    use crate::engine::Rule;

    fn apply(text: &str) -> String {
        Tags.apply(text, &IgnoreRanges::compute(text))
    }

    #[test]
    fn moves_pure_tag_line_under_heading_sorted() {
        assert_eq!(
            apply("# Groceries\nMilk.\n#shopping #errands"),
            "# Groceries\n#errands #shopping\nMilk."
        );
    }

    #[test]
    fn moves_to_top_when_no_heading() {
        assert_eq!(apply("Body.\n#note"), "#note\nBody.");
    }

    #[test]
    fn splits_mixed_line_in_place() {
        assert_eq!(
            apply("#idea a feature #todo write it"),
            "#idea a feature\n#todo write it"
        );
    }

    #[test]
    fn strips_redundant_closing_hash() {
        assert_eq!(apply("#project#"), "#project");
    }

    #[test]
    fn leaves_body_tags_alone() {
        assert_eq!(apply("see #ref in text"), "see #ref in text");
    }

    #[test]
    fn is_idempotent() {
        let once = apply("# T\nbody\n#b #a\n#c stuff #d more");
        assert_eq!(apply(&once), once);
    }
}
