# Claude Code — bear-formatter

A **formatter** for Bear notes — a pure `Markdown → Markdown` transform, like
gofmt for your notes. Pure Rust, single binary (Homebrew). Reads Bear's SQLite
database read-only; write-back goes through Bear's CLI.

**Not a linter.** There is no check mode, no "violation", no "unfixable"
concept, no rule codes. Every rule is a total transformation that always
produces canonical output. If a future concern is genuinely advisory (e.g.
"this note has no title"), it belongs in a different tool — don't smuggle lint
concepts back in here.

## Commands

| Task | Command |
|---|---|
| **Full pipeline** (format → lint → check → test) | `make` |
| **Format the Rust source** | `make format` (`cargo fmt`) |
| **Lint the Rust source** | `make lint` (`cargo fmt --check` + `cargo clippy -D warnings`) |
| **Build check** | `make check` (`cargo check`) |
| **Test** | `make test` (`cargo test`) |
| **Release binary** | `make build` |

`make lint`/`check` are about *our* Rust code quality (clippy) — not a mode of
the tool. "lint" appears nowhere in the tool's own vocabulary.

## Layout

- `src/engine/` — the formatting engine. `mod.rs` (`Formatter`, the `Rule`
  trait), `ignore.rs` (code spans/blocks rules must not touch).
- `src/rules/` — the rule catalog. `all()` returns every rule, in order. One
  demonstrator ships today (`final-newline`); the real catalog is TBD.
- `src/bear/` — read-only SQLite reader and note selectors (note / tag / all).
- `src/config.rs` — `bear-formatter.toml` discovery.
- `src/cli.rs` — clap CLI and output.
- `tests/` — contract tests through the public API.
- `axioms/` — submodule of shared engineering axioms (chasefinch/axioms). Follow
  the Code (X2xx) and Tests (X3xx) sections; there is no Rust section yet.

## How it works

- **Sequential rewrite**: `Formatter::format` runs each rule in turn, feeding
  one rule's output into the next. A rule is `fn(&str, &IgnoreRanges) -> String`
  — total and pure. No edit/patch bookkeeping; the diff for write-back is simply
  input vs. output.
- **Idempotence is the contract**: `format(format(x)) == format(x)`. Every rule
  must preserve it; `tests/integration.rs` checks it.
- **Ignore ranges**: `IgnoreRanges::compute` (via `pulldown-cmark` offsets)
  marks fenced/indented code and inline code. Content rules must consult it
  before touching a span — two trailing spaces are a hard break, a `#` inside
  code is not a heading. Recomputed each pass (cheap at note scale; optimize
  only if a profile says so — X600).
- **Read-only Bear**: opened `SQLITE_OPEN_READ_ONLY`; canonical text is
  `ZSFNOTE.ZTEXT`. Trashed / deleted / encrypted notes are skipped. Write-back
  (via `bearcli --no-update-modified`) is deliberately not wired yet — until it
  is, database targets read and report what *would* change; `--code` is the live
  path.

## Adding a rule

1. New module under `src/rules/`, a unit struct implementing `Rule`.
2. Register it in `rules::all()` (order matters).
3. Unit-test it, and confirm idempotence.
4. When a rule has rich multi-line before/after, reach for snapshot tests
   (`insta`) and add the dev-dependency then — the current demonstrator is too
   small to warrant it.

## Gotchas

- `PathBuf` has no `Display`; in `thiserror` messages use `{}` with
  `.path.display()`.
- Empty SQL params: `rusqlite::params![]`.
