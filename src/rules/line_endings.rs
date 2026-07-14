//! **line-endings** — normalize CRLF and lone CR to LF so later rules can
//! assume `\n`.

use crate::engine::ignore::IgnoreRanges;
use crate::engine::Rule;

pub struct LineEndings;

impl Rule for LineEndings {
    fn name(&self) -> &'static str {
        "line-endings"
    }

    fn apply(&self, text: &str, _ignore: &IgnoreRanges) -> String {
        text.replace("\r\n", "\n").replace('\r', "\n")
    }
}

#[cfg(test)]
mod tests {
    use super::LineEndings;
    use crate::engine::ignore::IgnoreRanges;
    use crate::engine::Rule;

    fn apply(text: &str) -> String {
        LineEndings.apply(text, &IgnoreRanges::compute(text))
    }

    #[test]
    fn converts_crlf_and_cr() {
        assert_eq!(apply("a\r\nb\rc\n"), "a\nb\nc\n");
    }
}
