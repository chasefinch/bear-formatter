//! Contract tests exercised through the public library API.

use bear_formatter::engine::Formatter;
use bear_formatter::rules;

fn format(text: &str) -> String {
    Formatter::new(rules::all()).format(text)
}

#[test]
fn adds_a_missing_final_newline() {
    assert_eq!(format("hello"), "hello\n");
}

#[test]
fn collapses_extra_trailing_newlines() {
    assert_eq!(format("hello\n\n\n"), "hello\n");
}

#[test]
fn leaves_well_formed_text_untouched() {
    let text = "# Title\n\nBody.\n";
    assert_eq!(format(text), text);
}

/// The defining property of a formatter: running it twice changes nothing the
/// first run didn't.
#[test]
fn is_idempotent() {
    let once = format("messy note\n\n\n");
    assert_eq!(format(&once), once);
}
