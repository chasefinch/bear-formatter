//! **layout** — the final pass: one blank line around every block, no leading
//! or trailing blanks, list indentation converted to tabs, and the list
//! blank-line rules.
//!
//! Blank lines are regenerated from block structure:
//! - one blank line between differing blocks (headings, paragraphs, code,
//!   quotes, tables, rules, lists);
//! - none between consecutive same-kind list items (bullets and todos are one
//!   kind; a numbered list next to them is a different list, so it gets a blank);
//! - none between a heading and a tag line that follows it (one after);
//! - inside a list, a blank between an item and its continuation paragraph is
//!   kept, and a multi-paragraph item is followed by a blank before the next
//!   item; a blank before a nested sub-list is dropped (a sub-list is part of
//!   its item, not a new paragraph) — unless a continuation paragraph already
//!   separates the marker from the sub-list, in which case a blank is emitted
//!   before the sub-list too, symmetric with the item's trailing blank.
//!
//! Known v1 gaps (to revisit): continuation-line indentation is left as-is
//! rather than retabbed, and blank lines are emitted empty (not indented to the
//! list level).

use crate::engine::ignore::IgnoreRanges;
use crate::engine::Rule;
use crate::rules::support::{heading_level, is_ordered_item, list_item_indent, starts_with_tag};

pub struct Layout;

#[derive(Clone, Copy, PartialEq)]
enum Group {
    Heading,
    Tag,
    ListItem,
    ListCont,
    Quote,
    Table,
    Rule,
    Label,
    Wikilink,
    Code,
    Para,
}

impl Rule for Layout {
    fn name(&self) -> &'static str {
        "layout"
    }

    fn apply(&self, text: &str, ignore: &IgnoreRanges) -> String {
        let mut lines: Vec<String> = Vec::new();
        let mut previous: Option<Group> = None;
        let mut had_blank = false;
        let mut hard_break = false;
        let mut levels: Vec<Level> = Vec::new();
        let mut start = 0;
        for piece in text.split_inclusive('\n') {
            let content = piece.strip_suffix('\n').unwrap_or(piece);
            let is_code = ignore.contains(start);
            start += piece.len();

            if !is_code && content.trim().is_empty() {
                had_blank = true;
                continue;
            }
            // An empty `>` line is a blank within a blockquote; drop it and let
            // the quote-splitting logic regenerate separators (keeps it idempotent).
            if !is_code && is_quote_blank(content) {
                continue;
            }

            let group = classify(content, is_code);
            let previous_is_list = matches!(previous, Some(Group::ListItem | Group::ListCont));
            let (rendered, list_blank) = match group {
                Group::ListItem => place_item(content, &mut levels, had_blank),
                Group::ListCont => place_cont(content, &mut levels, had_blank),
                _ => {
                    levels.clear();
                    (content.to_string(), false)
                }
            };

            if let Some(prev) = previous {
                let current_is_list = matches!(group, Group::ListItem | Group::ListCont);
                let (count, separator) = if previous_is_list && current_is_list {
                    (usize::from(list_blank), "")
                } else {
                    gap(prev, group, hard_break, &rendered, had_blank)
                };
                for _ in 0..count {
                    lines.push(separator.to_string());
                }
            }
            hard_break = rendered.ends_with("  ");
            lines.push(rendered);
            previous = Some(group);
            had_blank = false;
        }

        // A note should not end with a horizontal rule (or one trailed by
        // blanks). Only a real thematic break is stripped, never code content.
        if previous == Some(Group::Rule) {
            lines.pop();
            while lines.last().is_some_and(|line| line.trim().is_empty()) {
                lines.pop();
            }
        }

        let joined = lines.join("\n");
        if text.ends_with('\n') {
            format!("{joined}\n")
        } else {
            joined
        }
    }
}

fn classify(content: &str, is_code: bool) -> Group {
    if is_code {
        return Group::Code;
    }
    if heading_level(content).is_some() {
        return Group::Heading;
    }
    if starts_with_tag(content) {
        return Group::Tag;
    }
    if list_item_indent(content).is_some() {
        return Group::ListItem;
    }
    if content.starts_with([' ', '\t']) {
        return Group::ListCont;
    }
    let trimmed = content.trim();
    if trimmed.starts_with('>') {
        Group::Quote
    } else if trimmed.starts_with('|') {
        Group::Table
    } else if is_thematic_break(trimmed) {
        Group::Rule
    } else if is_label_line(trimmed) {
        Group::Label
    } else if is_wikilink_only(trimmed) {
        Group::Wikilink
    } else {
        Group::Para
    }
}

/// The separator to place before `current`: how many lines, and their text.
/// Blockquote paragraphs are split with an empty `>` line; everything else uses
/// blank lines.
fn gap(
    previous: Group,
    current: Group,
    prev_hard_break: bool,
    current_line: &str,
    had_blank: bool,
) -> (usize, &'static str) {
    if let (Group::Quote, Group::Quote) = (previous, current) {
        // Inside a blockquote a newline is a paragraph break too (split with an
        // empty `>` line), unless the line is a pipe-quote or the previous had a
        // hard break.
        let inner = quote_inner(current_line);
        let keep_together = prev_hard_break || inner.starts_with('|');
        return (usize::from(!keep_together), ">");
    }
    (
        desired_blanks(previous, current, prev_hard_break, had_blank),
        "",
    )
}

/// The content of a blockquote line after its `>` marker.
fn quote_inner(line: &str) -> &str {
    let after_marker = line.trim_start().strip_prefix('>').unwrap_or(line);
    after_marker.strip_prefix(' ').unwrap_or(after_marker)
}

/// Whether `content` is an empty blockquote line (`>` with nothing after it).
fn is_quote_blank(content: &str) -> bool {
    content
        .trim_start()
        .strip_prefix('>')
        .is_some_and(|rest| rest.trim().is_empty())
}

/// Whether the whole line is a single wikilink, e.g. `[[Page]]`.
fn is_wikilink_only(trimmed: &str) -> bool {
    trimmed
        .strip_prefix("[[")
        .and_then(|inner| inner.strip_suffix("]]"))
        .is_some_and(|inner| !inner.is_empty() && !inner.contains("[[") && !inner.contains("]]"))
}

/// Whether the whole line is a single strong-emphasis span, e.g. `**Label:**`.
/// Such a line reads as its own block, like a small heading, so it gets a blank
/// line around it rather than being glued to the paragraph below.
fn is_label_line(trimmed: &str) -> bool {
    ["**", "__"].iter().any(|delimiter| {
        trimmed
            .strip_prefix(delimiter)
            .and_then(|inner| inner.strip_suffix(delimiter))
            .is_some_and(|inner| !inner.is_empty() && !inner.contains(delimiter))
    })
}

/// How many blank lines belong between two adjacent blocks.
///
/// In Bear a single newline is a paragraph break (the editor wraps text; you
/// never hand-break inside a paragraph), so consecutive prose lines are split
/// with a blank line — unless the previous line ends with an explicit two-space
/// hard break.
fn desired_blanks(
    previous: Group,
    current: Group,
    prev_hard_break: bool,
    had_blank: bool,
) -> usize {
    match (previous, current) {
        (Group::Heading, Group::Tag) | (Group::Tag, Group::Tag) => 0,
        (Group::Code, Group::Code) => 0,
        // Consecutive wikilink-only lines read as a table of contents.
        (Group::Wikilink, Group::Wikilink) => 0,
        // Contiguous pipe lines are one block (never split by a paragraph
        // break), but a blank in the source separates two distinct tables and
        // is kept — deliberate merging is the tables rule's job.
        (Group::Table, Group::Table) => usize::from(had_blank),
        (Group::Para, Group::Para) => usize::from(!prev_hard_break),
        _ => 1,
    }
}

/// A list kind: bullets and todos are `Bullet`; numbered items are `Ordered`.
/// A numbered list beside a bulleted one is a *different* list.
#[derive(Clone, Copy, PartialEq)]
enum ListKind {
    Bullet,
    Ordered,
}

fn list_kind(content: &str) -> ListKind {
    if is_ordered_item(content) {
        ListKind::Ordered
    } else {
        ListKind::Bullet
    }
}

/// One open list level: its indent width, whether the current item is loose
/// (multi-paragraph), and its list kind.
struct Level {
    width: usize,
    loose: bool,
    kind: ListKind,
}

/// Place a list-item line: adjust the level stack and return the retabbed line
/// plus whether a blank should precede it. A blank precedes a sibling when the
/// previous item here was loose (multi-paragraph) or when the list kind changes
/// (a numbered list next to bullets/todos). Nesting adds none, unless the parent
/// item has gone loose (a continuation paragraph precedes the sub-list).
fn place_item(content: &str, levels: &mut Vec<Level>, had_blank: bool) -> (String, bool) {
    let width = list_item_indent(content).unwrap_or(0);
    let kind = list_kind(content);
    while levels.last().is_some_and(|level| width < level.width) {
        levels.pop();
    }
    let going_deeper = match levels.last() {
        None => true,
        Some(level) => width > level.width,
    };
    let blank_before = if going_deeper {
        // A nested sub-list is part of its parent item, not a new paragraph —
        // a blank before it is dropped. (A blank before continuation *prose*
        // is a real paragraph break and is kept, in `place_cont`.) The exception
        // is a parent that has already gone loose: a continuation paragraph
        // sits between the marker and this sub-list, so the sub-list follows a
        // real paragraph and the blank is kept — symmetric with the trailing one.
        let parent_loose = levels.last().is_some_and(|level| level.loose);
        let nested = !levels.is_empty();
        levels.push(Level {
            width,
            loose: false,
            kind,
        });
        // A nested sub-list hugs its marker (no blank) unless the parent has gone
        // loose, in which case the blank is emitted structurally — like the
        // trailing one — whether or not the source had it. A root list opening
        // here has its spacing decided by `gap` instead, so `had_blank` is moot.
        if nested {
            parent_loose
        } else {
            had_blank
        }
    } else {
        match levels.last_mut() {
            Some(level) => {
                let blank = level.loose || level.kind != kind;
                level.loose = false;
                level.kind = kind;
                blank
            }
            None => false,
        }
    };
    let depth = levels.len().saturating_sub(1);
    let rest = content.trim_start_matches([' ', '\t']);
    (format!("{}{}", "\t".repeat(depth), rest), blank_before)
}

/// Place a list continuation line (indented text): it belongs to the deepest
/// open item, which a preceding blank marks loose. Its blank-before is preserved.
fn place_cont(content: &str, levels: &mut [Level], had_blank: bool) -> (String, bool) {
    if had_blank {
        if let Some(owner) = levels.last_mut() {
            owner.loose = true;
        }
    }
    let depth = levels.len();
    let rest = content.trim_start_matches([' ', '\t']);
    (format!("{}{}", "\t".repeat(depth), rest), had_blank)
}

fn is_thematic_break(trimmed: &str) -> bool {
    trimmed.len() >= 3
        && (trimmed.chars().all(|character| character == '-')
            || trimmed.chars().all(|character| character == '*')
            || trimmed.chars().all(|character| character == '_'))
}

#[cfg(test)]
mod tests {
    use super::Layout;
    use crate::engine::ignore::IgnoreRanges;
    use crate::engine::Rule;

    fn apply(text: &str) -> String {
        Layout.apply(text, &IgnoreRanges::compute(text))
    }

    #[test]
    fn one_blank_between_blocks_and_none_leading() {
        assert_eq!(apply("\n\n# Title\nBody"), "# Title\n\nBody");
    }

    #[test]
    fn collapses_multiple_blanks() {
        assert_eq!(apply("a\n\n\n\nb"), "a\n\nb");
    }

    #[test]
    fn no_blanks_between_list_items() {
        assert_eq!(apply("- a\n\n- b\n- c"), "- a\n- b\n- c");
    }

    #[test]
    fn blank_around_root_list() {
        assert_eq!(apply("Text\n- a\n- b\nMore"), "Text\n\n- a\n- b\n\nMore");
    }

    #[test]
    fn indents_nested_items_with_tabs() {
        assert_eq!(apply("- a\n  - b\n    - c"), "- a\n\t- b\n\t\t- c");
    }

    #[test]
    fn no_blank_between_heading_and_tag_line() {
        assert_eq!(apply("# T\n\n#a #b\n\nBody"), "# T\n#a #b\n\nBody");
    }

    #[test]
    fn multi_paragraph_item_keeps_and_follows_with_blank() {
        let input = "- one\n- two\n\n  more\n- three";
        assert_eq!(apply(input), "- one\n- two\n\n\tmore\n\n- three");
    }

    #[test]
    fn bold_label_line_becomes_its_own_block() {
        assert_eq!(apply("**Label:**\ntext"), "**Label:**\n\ntext");
        assert_eq!(apply("para\n**Note**\nmore"), "para\n\n**Note**\n\nmore");
    }

    #[test]
    fn tag_led_line_hugs_the_heading() {
        // A tag with a trailing date is still a tag line — no blank above it.
        assert_eq!(
            apply("# T\n#media/meeting 1/28/23\nBody"),
            "# T\n#media/meeting 1/28/23\n\nBody"
        );
    }

    #[test]
    fn single_newline_becomes_a_paragraph_break() {
        assert_eq!(
            apply("one thought\nanother thought"),
            "one thought\n\nanother thought"
        );
    }

    #[test]
    fn hard_break_stays_attached() {
        assert_eq!(apply("line one  \nline two"), "line one  \nline two");
    }

    #[test]
    fn strips_a_trailing_horizontal_rule() {
        assert_eq!(apply("# T\nbody\n\n---"), "# T\n\nbody");
        assert_eq!(apply("body\n---\n"), "body\n");
    }

    #[test]
    fn pipe_lines_stay_contiguous_with_a_blank_after() {
        assert_eq!(
            apply("| John Smith\n| 123 Main St\n| Springfield"),
            "| John Smith\n| 123 Main St\n| Springfield"
        );
        // A blank between pipe blocks separates them (two tables stay two
        // tables); the blank after the block is still enforced.
        assert_eq!(apply("| a\n\n| b\ntext"), "| a\n\n| b\n\ntext");
    }

    #[test]
    fn blockquote_lines_split_into_paragraphs() {
        assert_eq!(apply("> one\n> two"), "> one\n>\n> two");
        // Pipe-quote lines stay together.
        assert_eq!(apply("> | a\n> | b"), "> | a\n> | b");
        let once = apply("> one\n> two\n> three");
        assert_eq!(apply(&once), once);
    }

    #[test]
    fn consecutive_wikilinks_are_a_toc() {
        assert_eq!(apply("[[A]]\n[[B]]\n[[C]]"), "[[A]]\n[[B]]\n[[C]]");
        assert_eq!(apply("[[A]]\n\n[[B]]"), "[[A]]\n[[B]]");
    }

    #[test]
    fn loose_list_item_gets_a_trailing_blank() {
        // A multi-paragraph item (nested content separated by a blank) is
        // followed by a blank before the next sibling — like the blank before it.
        // Because a continuation paragraph precedes the sub-list here, the blank
        // before the sub-list is kept too (it follows a real paragraph).
        let input = "- a\n- b\n\n\tnote\n\n\t- x\n\t- y\n- c";
        assert_eq!(apply(input), "- a\n- b\n\n\tnote\n\n\t- x\n\t- y\n\n- c");
        let once = apply(input);
        assert_eq!(apply(&once), once);
    }

    #[test]
    fn continuation_paragraph_keeps_blank_before_sub_list() {
        // Marker → paragraph → sub-list: the sub-list follows a real paragraph,
        // so blanks land on both sides of it (unlike a sub-list glued straight
        // to the marker, below).
        let input = "- item\n\n\t*lead-in:*\n\t- one\n\t- two\n- next";
        assert_eq!(
            apply(input),
            "- item\n\n\t*lead-in:*\n\n\t- one\n\t- two\n\n- next"
        );
        let once = apply(input);
        assert_eq!(apply(&once), once);
    }

    #[test]
    fn no_blank_between_an_item_and_its_sub_list() {
        assert_eq!(
            apply("3. Habit\n\n\t- Kid Time\n\t- Phone Use"),
            "3. Habit\n\t- Kid Time\n\t- Phone Use"
        );
        assert_eq!(apply("- a\n\n\t- x\n- b"), "- a\n\t- x\n- b");
    }

    #[test]
    fn numbered_list_is_separate_from_bullets() {
        // Bullets and todos are one list (blank between them removed); a numbered
        // list is a different one and gets a blank.
        assert_eq!(
            apply("- a\n\n- [ ] b\n1. c\n2. d"),
            "- a\n- [ ] b\n\n1. c\n2. d"
        );
    }

    #[test]
    fn numbered_nested_in_bullets_has_no_extra_spacing() {
        assert_eq!(
            apply("- a\n\t1. x\n\t2. y\n- b"),
            "- a\n\t1. x\n\t2. y\n- b"
        );
    }

    #[test]
    fn is_idempotent() {
        let once = apply("# T\n#x\nintro\n- a\n  - b\n\n  cont\n- c\ntail\n**Label:**\nx");
        assert_eq!(apply(&once), once);
    }
}
