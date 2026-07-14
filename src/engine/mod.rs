//! The formatting engine.
//!
//! A [`Formatter`] runs an ordered set of [`Rule`]s over a note's Markdown.
//! Every rule is a *total* transformation: given the current text (and the
//! [`ignore::IgnoreRanges`] it must leave untouched) it returns the text
//! rewritten into canonical form. There is no "violation" and no check mode —
//! this is a formatter, like gofmt, not a linter.

pub mod ignore;

use crate::engine::ignore::IgnoreRanges;

/// A single formatting rule: a total transformation over a note's Markdown.
///
/// Rules must not alter the [`IgnoreRanges`] regions (fenced/indented code and
/// inline code); everything else is theirs to normalize. Every rule must be
/// idempotent — applying it twice matches applying it once.
pub trait Rule {
    /// A short, kebab-case identifier, used in configuration and diagnostics.
    fn name(&self) -> &'static str;

    /// Return `text` rewritten into canonical form.
    fn apply(&self, text: &str, ignore: &IgnoreRanges) -> String;
}

/// Runs an ordered catalog of rules over Markdown.
pub struct Formatter {
    rules: Vec<Box<dyn Rule>>,
}

impl Formatter {
    /// Build a formatter from `rules`, applied in order.
    pub fn new(rules: Vec<Box<dyn Rule>>) -> Self {
        Self { rules }
    }

    /// Format `text` by applying every rule in turn.
    ///
    /// Ignore ranges are recomputed before each rule so a rule always sees the
    /// output of the one before it. This keeps rules independent and the whole
    /// pipeline idempotent.
    pub fn format(&self, text: &str) -> String {
        let mut current = text.to_string();
        for rule in &self.rules {
            let ignore = IgnoreRanges::compute(&current);
            current = rule.apply(&current, &ignore);
        }
        current
    }
}
