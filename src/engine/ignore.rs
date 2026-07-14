//! Protected byte ranges that rules must not modify.

use std::ops::Range;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// Byte ranges within a document that rules must leave untouched — fenced and
/// indented code blocks and inline code spans.
///
/// Content rules consult this to avoid corrupting literal text: two trailing
/// spaces inside a code block are significant, a `#` inside code is not a
/// heading, and so on.
pub struct IgnoreRanges {
    ranges: Vec<Range<usize>>,
}

impl IgnoreRanges {
    /// Compute the protected ranges for `text`.
    pub fn compute(text: &str) -> Self {
        let mut ranges = Vec::new();
        let mut block_start = None;
        for (event, range) in Parser::new_ext(text, Options::empty()).into_offset_iter() {
            match event {
                Event::Start(Tag::CodeBlock(_)) => block_start = Some(range.start),
                Event::End(TagEnd::CodeBlock) => {
                    if let Some(start) = block_start.take() {
                        ranges.push(start..range.end);
                    }
                }
                Event::Code(_) => ranges.push(range),
                _ => {}
            }
        }
        Self { ranges }
    }

    /// Whether `offset` falls inside a protected range.
    pub fn contains(&self, offset: usize) -> bool {
        self.ranges.iter().any(|range| range.contains(&offset))
    }

    /// Whether any part of `span` overlaps a protected range.
    pub fn overlaps(&self, span: &Range<usize>) -> bool {
        self.ranges
            .iter()
            .any(|range| span.start < range.end && range.start < span.end)
    }
}

#[cfg(test)]
mod tests {
    use super::IgnoreRanges;

    #[test]
    fn protects_fenced_code_blocks() {
        let text = "before\n\n```\ncode #1\n```\n\nafter";
        let ranges = IgnoreRanges::compute(text);
        let inside = text.find("code").expect("fixture contains code");
        assert!(ranges.contains(inside));
        assert!(!ranges.contains(0));
    }

    #[test]
    fn protects_inline_code() {
        let text = "a `snippet` here";
        let ranges = IgnoreRanges::compute(text);
        let inside = text.find("snippet").expect("fixture contains snippet");
        assert!(ranges.contains(inside));
    }
}
