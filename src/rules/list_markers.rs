//! **list-markers** — unordered bullets become `-`, and a marker followed by any
//! run of spaces or a tab (as pastes often produce, e.g. `•\t`) is normalized to
//! exactly one space. Empty items are dropped, and duplicated markers (from
//! pastes) are collapsed. Ordered numbers are left as they are — no renumbering.

use crate::engine::ignore::IgnoreRanges;
use crate::engine::Rule;
use crate::rules::support::BULLETS;

pub struct ListMarkers;

impl Rule for ListMarkers {
    fn name(&self) -> &'static str {
        "list-markers"
    }

    fn apply(&self, text: &str, ignore: &IgnoreRanges) -> String {
        let mut out = String::with_capacity(text.len());
        let mut start = 0;
        for piece in text.split_inclusive('\n') {
            let has_newline = piece.ends_with('\n');
            let content = piece.strip_suffix('\n').unwrap_or(piece);
            if ignore.contains(start) {
                out.push_str(content);
                if has_newline {
                    out.push('\n');
                }
            } else if let Some(rendered) = reformat(content) {
                out.push_str(&rendered);
                if has_newline {
                    out.push('\n');
                }
            }
            // A `None` means an empty list item — drop the line and its newline.
            start += piece.len();
        }
        out
    }
}

/// Reformat one line, or `None` if it is an empty list item to drop.
fn reformat(line: &str) -> Option<String> {
    let indent_len = line.len() - line.trim_start_matches([' ', '\t']).len();
    let (indent, rest) = line.split_at(indent_len);

    let Some((_, mut content)) = split_marker(rest) else {
        return Some(line.to_string());
    };
    while let Some((_, inner)) = split_marker(content) {
        content = inner;
    }
    let content = content.trim_end();
    if content.is_empty() {
        return None;
    }
    let marker = ordered_marker(rest).map_or_else(|| "-".to_string(), str::to_string);
    Some(format!("{indent}{marker} {content}"))
}

/// If `rest` begins with a list marker, return `(marker, content after it)`.
fn split_marker(rest: &str) -> Option<(&str, &str)> {
    if let Some(marker) = ordered_marker(rest) {
        let after = &rest[marker.len()..];
        if after.is_empty() || after.starts_with([' ', '\t']) {
            return Some((marker, after.trim_start_matches([' ', '\t'])));
        }
        return None;
    }
    let first = rest.chars().next()?;
    if BULLETS.contains(&first) {
        let after = &rest[first.len_utf8()..];
        if after.is_empty() || after.starts_with([' ', '\t']) {
            return Some((
                &rest[..first.len_utf8()],
                after.trim_start_matches([' ', '\t']),
            ));
        }
    }
    // A bare `.` followed by a tab is a pasted marker (a numbered item that
    // lost its number). A space is not enough — ". . ." and shell `. script`
    // lines are prose.
    if first == '.' && rest[1..].starts_with('\t') {
        return Some((&rest[..1], rest[1..].trim_start_matches([' ', '\t'])));
    }
    None
}

/// If `rest` starts with an ordered marker (digits then `.` or `)`), return it.
fn ordered_marker(rest: &str) -> Option<&str> {
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    match rest.as_bytes().get(digits) {
        Some(b'.' | b')') => Some(&rest[..=digits]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::ListMarkers;
    use crate::engine::ignore::IgnoreRanges;
    use crate::engine::Rule;

    fn apply(text: &str) -> String {
        ListMarkers.apply(text, &IgnoreRanges::compute(text))
    }

    #[test]
    fn normalizes_bullets_to_dash() {
        assert_eq!(apply("* a\n+ b\n• c"), "- a\n- b\n- c");
    }

    #[test]
    fn normalizes_more_bullet_glyphs() {
        assert_eq!(apply("◦ a\n▪ b\n‣ c\n⁃ d\n· e"), "- a\n- b\n- c\n- d\n- e");
    }

    #[test]
    fn one_space_after_marker() {
        assert_eq!(apply("-    x"), "- x");
    }

    #[test]
    fn accepts_a_tab_after_the_marker() {
        assert_eq!(apply("•\tFoo\n-\tbar\n1.\tbaz"), "- Foo\n- bar\n1. baz");
    }

    #[test]
    fn accepts_a_tab_after_a_nested_marker() {
        assert_eq!(apply("\t•\tBar"), "\t- Bar");
    }

    #[test]
    fn a_bare_period_with_a_tab_is_a_pasted_marker() {
        assert_eq!(apply(".\t**Roasted Carrots**"), "- **Roasted Carrots**");
        // With a space (or nothing) after it, a period is prose.
        assert_eq!(apply(". . ."), ". . .");
        assert_eq!(apply("."), ".");
    }

    #[test]
    fn is_idempotent_with_tabs() {
        let once = apply("•\ta\n\t•\tb\n1.\tc");
        assert_eq!(apply(&once), once);
    }

    #[test]
    fn preserves_todos_and_ordered_numbers() {
        assert_eq!(apply("- [ ] task\n3. third"), "- [ ] task\n3. third");
    }

    #[test]
    fn drops_empty_items_and_collapses_duplicates() {
        assert_eq!(apply("- \n- - real"), "- real");
    }

    #[test]
    fn leaves_emphasis_alone() {
        assert_eq!(apply("*emphasis* text"), "*emphasis* text");
    }

    #[test]
    fn is_idempotent() {
        let once = apply("*  a\n- - b\n1.   c");
        assert_eq!(apply(&once), once);
    }
}
