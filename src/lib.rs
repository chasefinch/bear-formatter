//! bear-formatter — a formatter for Bear notes. 🐻
//!
//! Bear keeps each note's canonical Markdown in a SQLite database. This crate
//! reads that Markdown (read-only) and rewrites it into a consistent, tidy
//! form — a pure `Markdown → Markdown` transform, like gofmt for your notes.
//! Writing the result back into Bear goes through Bear's own CLI and is
//! deliberately kept out of the read path.
//!
//! The public surface is intentionally small:
//!
//! - [`engine`] — the formatting engine: [`engine::Formatter`], the
//!   [`engine::Rule`] trait, and the ignore-range machinery.
//! - [`rules`] — the rule catalog ([`rules::all`]).
//! - [`bear`] — the read-only Bear database reader.
//! - [`config`] — `bear-formatter.toml` discovery.
//! - [`cli`] — wires the above together for the binary.

pub mod bear;
pub mod cli;
pub mod config;
pub mod engine;
pub mod rules;
