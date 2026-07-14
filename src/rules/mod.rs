//! The rule catalog.
//!
//! Only the demonstrator rule ships today; the real catalog is defined once the
//! rule set is agreed. [`all`] returns every rule the formatter runs, in the
//! order they are applied.

mod final_newline;

pub use final_newline::FinalNewline;

use crate::engine::Rule;

/// Every rule in the catalog, in application order.
pub fn all() -> Vec<Box<dyn Rule>> {
    vec![Box::new(FinalNewline)]
}
