//! **tags** — Bear tags, tidied.
//!
//! A line that *starts* with a tag is metadata: it is gathered and moved to the
//! top — right under the first heading, or the very top if there is none. Bare
//! tags are merged, deduped, and sorted onto one line; a tag carrying trailing
//! text (like a meeting date) keeps that text on its own line. Multiple tags on
//! one line are split so each keeps the text that follows it. A redundant
//! closing `#` is stripped. Tags mid-prose (not at the line start) and tags
//! inside code are left where they are.

use std::collections::HashSet;

use crate::engine::ignore::IgnoreRanges;
use crate::engine::Rule;
use crate::rules::support::{heading_level, starts_with_tag};

pub struct Tags;

enum Classified {
    Verbatim { text: String, heading: bool },
    Removed,
}

impl Rule for Tags {
    fn name(&self) -> &'static str {
        "tags"
    }

    fn apply(&self, text: &str, ignore: &IgnoreRanges) -> String {
        let mut classified = Vec::new();
        let mut bare = Vec::new();
        let mut annotated = Vec::new();
        let mut start = 0;
        for piece in text.split_inclusive('\n') {
            let content = piece.strip_suffix('\n').unwrap_or(piece);
            if ignore.contains(start) {
                classified.push(Classified::Verbatim {
                    text: content.to_string(),
                    heading: false,
                });
            } else if starts_with_tag(content) {
                for entry in split_mixed(content) {
                    if entry.split_whitespace().count() > 1 {
                        annotated.push(entry);
                    } else {
                        bare.push(entry);
                    }
                }
                classified.push(Classified::Removed);
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
                Classified::Removed => {}
            }
        }

        // The top block: merged bare tags on one line, then annotated lines.
        let mut top = Vec::new();
        if !bare.is_empty() {
            top.push(merge_and_sort(bare));
        }
        top.extend(annotated);

        let insert_at = anchor.map_or(0, |index| index + 1);
        for (offset, line) in top.into_iter().enumerate() {
            lines.insert(insert_at + offset, line);
        }

        let joined = lines.join("\n");
        if text.ends_with('\n') {
            format!("{joined}\n")
        } else {
            joined
        }
    }
}

/// Split a tag-led line so each tag starts a new entry carrying the text after
/// it, preserving the original indentation.
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
    fn moves_a_tag_with_a_date_to_the_top() {
        assert_eq!(
            apply("# Dialpad\nnotes here\n#media/meeting 4/10/2023"),
            "# Dialpad\n#media/meeting 4/10/2023\nnotes here"
        );
    }

    #[test]
    fn splits_a_multi_tag_line() {
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
