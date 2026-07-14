//! Binary entry point.
//!
//! All logic lives in the library so it can be tested through public
//! interfaces (see `tests/`).

use std::process::ExitCode;

fn main() -> ExitCode {
    bear_formatter::cli::run()
}
