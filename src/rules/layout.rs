//! **layout** — the final pass: one blank line around every block, no leading
//! or trailing blanks, list indentation converted to tabs, and the list
//! blank-line rules.
//!
//! Blank lines are regenerated from block structure:
//! - one blank line between differing blocks (headings, paragraphs, code,
//!   quotes, tables, rules, lists);
//! - none between consecutive list items;
//! - none between a heading and a tag line that follows it (one after);
//! - inside a list, a blank between an item and its continuation paragraph is
//!   kept, and a multi-paragraph item is followed by a blank before the next
//!   item.
//!
//! Known v1 gaps (to revisit): continuation-line indentation is left as-is
//! rather than retabbed, and blank lines are emitted empty (not indented to the
//! list level).

use crate::engine::ignore::IgnoreRanges;
use crate::engine::Rule;
use crate::rules::support::{heading_level, list_item_indent, starts_with_tag};

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
        let mut list_depths: Vec<usize> = Vec::new();
        let mut start = 0;
        for piece in text.split_inclusive('\n') {
            let content = piece.strip_suffix('\n').unwrap_or(piece);
            let is_code = ignore.contains(start);
            start += piece.len();

            if !is_code && content.trim().is_empty() {
                had_blank = true;
                continue;
            }

            let group = classify(content, is_code);
            let rendered = match group {
                Group::ListItem => retab_item(content, &mut list_depths),
                Group::ListCont => retab_continuation(content, &list_depths),
                _ => {
                    list_depths.clear();
                    content.to_string()
                }
            };

            if let Some(prev) = previous {
                for _ in 0..desired_blanks(prev, group, had_blank, hard_break) {
                    lines.push(String::new());
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
    } else {
        Group::Para
    }
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
    had_blank: bool,
    prev_hard_break: bool,
) -> usize {
    match (previous, current) {
        (Group::ListItem, Group::ListItem) => 0,
        (Group::ListItem | Group::ListCont, Group::ListCont) => usize::from(had_blank),
        (Group::ListCont, Group::ListItem) => 1,
        (Group::Heading, Group::Tag) | (Group::Tag, Group::Tag) => 0,
        (Group::Code, Group::Code)
        | (Group::Quote, Group::Quote)
        | (Group::Table, Group::Table) => 0,
        (Group::Para, Group::Para) => usize::from(!prev_hard_break),
        _ => 1,
    }
}

/// Retab a list-item line: update the depth stack and re-indent with tabs.
fn retab_item(content: &str, depths: &mut Vec<usize>) -> String {
    let width = list_item_indent(content).unwrap_or(0);
    while depths.last().is_some_and(|&top| width < top) {
        depths.pop();
    }
    let deeper = match depths.last() {
        None => true,
        Some(&top) => width > top,
    };
    if deeper {
        depths.push(width);
    }
    let depth = depths.len().saturating_sub(1);
    let rest = content.trim_start_matches([' ', '\t']);
    format!("{}{}", "\t".repeat(depth), rest)
}

/// A continuation line keeps its text but is indented one level past the
/// current list depth.
fn retab_continuation(content: &str, depths: &[usize]) -> String {
    let depth = depths.len();
    let rest = content.trim_start_matches([' ', '\t']);
    format!("{}{}", "\t".repeat(depth), rest)
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
    fn is_idempotent() {
        let once = apply("# T\n#x\nintro\n- a\n  - b\n\n  cont\n- c\ntail\n**Label:**\nx");
        assert_eq!(apply(&once), once);
    }
}
