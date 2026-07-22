//! **typography** — curly quotes and apostrophes, and a proper ellipsis:
//! `...` and spaced `. . .` (three or more dots) both become `…`.
//! Code (fenced and inline) is left untouched.

use crate::engine::ignore::IgnoreRanges;
use crate::engine::Rule;

pub struct Typography;

impl Rule for Typography {
    fn name(&self) -> &'static str {
        "typography"
    }

    fn apply(&self, text: &str, ignore: &IgnoreRanges) -> String {
        let mut out = String::with_capacity(text.len());
        let mut previous: Option<char> = None;
        let mut cursor = 0;
        while let Some(current) = text[cursor..].chars().next() {
            if ignore.contains(cursor) {
                out.push(current);
                previous = Some(current);
                cursor += current.len_utf8();
                continue;
            }
            if text[cursor..].starts_with("...") {
                out.push('…');
                previous = Some('…');
                cursor += 3;
                continue;
            }
            if let Some(length) = spaced_ellipsis(&text[cursor..]) {
                out.push('…');
                previous = Some('…');
                cursor += length;
                continue;
            }
            let emitted = match current {
                '"' if opens(previous) => '“',
                '"' => '”',
                '\'' => single_quote(previous, text[cursor + 1..].chars().next()),
                other => other,
            };
            out.push(emitted);
            previous = Some(current);
            cursor += current.len_utf8();
        }
        out
    }
}

/// If `rest` opens with a spaced ellipsis — three or more dots separated only
/// by spaces or tabs, like `. . .` — the byte length of the whole run. Dots
/// with anything else between them (initials like `J. R. R.`) never match.
fn spaced_ellipsis(rest: &str) -> Option<usize> {
    let bytes = rest.as_bytes();
    if bytes.first() != Some(&b'.') {
        return None;
    }
    let mut dots = 1;
    let mut end = 1;
    let mut cursor = 1;
    loop {
        let mut probe = cursor;
        while bytes
            .get(probe)
            .is_some_and(|&byte| byte == b' ' || byte == b'\t')
        {
            probe += 1;
        }
        if probe > cursor && bytes.get(probe) == Some(&b'.') {
            dots += 1;
            end = probe + 1;
            cursor = probe + 1;
        } else {
            break;
        }
    }
    (dots >= 3).then_some(end)
}

/// Whether a quote here should be an opening one: at the start, or after
/// whitespace or an opening bracket.
fn opens(previous: Option<char>) -> bool {
    match previous {
        None => true,
        Some(character) => character.is_whitespace() || "([{".contains(character),
    }
}

fn single_quote(previous: Option<char>, next: Option<char>) -> char {
    let followed_by_word = next.is_some_and(|character| character.is_alphanumeric());
    if opens(previous) && followed_by_word {
        '‘'
    } else {
        '’'
    }
}

#[cfg(test)]
mod tests {
    use super::Typography;
    use crate::engine::ignore::IgnoreRanges;
    use crate::engine::Rule;

    fn apply(text: &str) -> String {
        Typography.apply(text, &IgnoreRanges::compute(text))
    }

    #[test]
    fn curls_quotes_and_apostrophes() {
        assert_eq!(apply(r#""hi," it's here"#), "“hi,” it’s here");
    }

    #[test]
    fn makes_ellipsis() {
        assert_eq!(apply("wait..."), "wait…");
    }

    #[test]
    fn collapses_spaced_ellipses() {
        assert_eq!(apply(". . ."), "…");
        assert_eq!(apply("so. . . yes"), "so… yes");
        // Wider spacing and a fourth dot are still one ellipsis.
        assert_eq!(apply(".  .  ."), "…");
        assert_eq!(apply(". . . ."), "…");
    }

    #[test]
    fn leaves_dot_pairs_and_initials_alone() {
        assert_eq!(apply(". ."), ". .");
        assert_eq!(apply("J. R. R. Tolkien"), "J. R. R. Tolkien");
    }

    #[test]
    fn leaves_code_alone() {
        assert_eq!(apply("`it's ...`"), "`it's ...`");
    }

    #[test]
    fn is_idempotent() {
        let once = apply(r#"She said "no"... it's fine"#);
        assert_eq!(apply(&once), once);
    }
}
