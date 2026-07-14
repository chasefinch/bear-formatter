//! The rule catalog.
//!
//! [`all`] returns every rule the formatter runs, in application order. Order
//! matters: text cleanups first, then structure (heading levels, tags), then
//! the final layout pass.

mod final_newline;
mod footnotes;
mod heading_levels;
mod headings;
mod layout;
mod line_endings;
mod list_markers;
mod support;
mod tags;
mod typography;
mod whitespace;

pub use final_newline::FinalNewline;
pub use footnotes::Footnotes;
pub use heading_levels::HeadingLevels;
pub use headings::Headings;
pub use layout::Layout;
pub use line_endings::LineEndings;
pub use list_markers::ListMarkers;
pub use tags::Tags;
pub use typography::Typography;
pub use whitespace::Whitespace;

use crate::engine::Rule;

/// Every rule in the catalog, in application order.
pub fn all() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(LineEndings),
        Box::new(Typography),
        Box::new(Whitespace),
        Box::new(Headings),
        Box::new(ListMarkers),
        Box::new(Footnotes),
        Box::new(Tags),
        Box::new(HeadingLevels),
        Box::new(Layout),
        Box::new(FinalNewline),
    ]
}
