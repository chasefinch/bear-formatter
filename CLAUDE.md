# Claude Code — bear-formatter

A **formatter** for Bear notes — a pure `Markdown → Markdown` transform, like
gofmt for your notes. Pure Rust, single binary. The installed command is
`bear-format`. Reads a Bear SQLite database read-only; writes changed notes back
through `bearcli overwrite`. Markdown files/globs are edited directly on disk.

**Not a linter.** No check mode, no "violation", no "unfixable", no rule codes.
Every rule is a total, idempotent transformation. Advisory concerns ("this note
has no title") belong in a different tool — don't smuggle lint concepts in here.

## Commands

| Task | Command |
|---|---|
| **Full pipeline** (format → lint → check → test) | `make` |
| **Format the Rust source** | `make format` (`cargo fmt`) |
| **Lint the Rust source** | `make lint` (`cargo fmt --check` + `cargo clippy -D warnings`) |
| **Build check** | `make check` (`cargo check`) |
| **Test** | `make test` (`cargo test`) |
| **Release binary** | `make build` (→ `target/release/bear-format`) |

`make lint`/`check` are about *our* Rust code quality (clippy), not a mode of the
tool.

## Layout

- `src/engine/` — the engine. `mod.rs` (`Formatter`, the `Rule` trait),
  `ignore.rs` (code spans/blocks rules must not touch).
- `src/rules/` — the rule catalog; `all()` returns every rule in application
  order (see `docs/rules.md`). `support.rs` holds shared parsing (heading / tag /
  list markers).
- `src/bear/` — SQLite reader (`BearDatabase`, read-only) plus `bearcli_path` and
  `overwrite_note` for write-back.
- `src/cli.rs` — clap CLI: positional paths (a Bear DB or Markdown globs),
  `--dry-run`, `--code`.
- `tests/` — contract tests through the public API.
- `axioms/` — submodule of shared engineering axioms (chasefinch/axioms). Follow
  the Code (X2xx) and Tests (X3xx) sections; there is no Rust section yet.

## How it works

- **Sequential rewrite**: `Formatter::format` runs each rule in turn, feeding one
  rule's output to the next. A rule is `fn(&str, &IgnoreRanges) -> String` — total
  and pure. The diff for write-back is simply input vs. output.
- **Idempotence is the contract**: `format(format(x)) == format(x)`, checked in
  `tests/integration.rs` and verified across real notes.
- **Ignore ranges**: `IgnoreRanges::compute` (pulldown-cmark offsets) marks
  fenced/indented code and inline code; content rules consult it before touching a
  span. Recomputed each pass (cheap at note scale — X600).
- **Bear I/O**: read `ZSFNOTE.ZTEXT` via `SQLITE_OPEN_READ_ONLY`; write each
  changed note with `bearcli overwrite <id> --no-update-modified`, content on
  stdin (verbatim — bearcli does not interpret escapes on stdin). Never write the
  SQLite file directly (CloudKit sync state, derived caches). The `--force`
  attachment gate is left OFF, so attachment-dropping writes are rejected.
  `ZUNIQUEIDENTIFIER` doubles as the bearcli note-id.

## Adding a rule

1. New module under `src/rules/`, a struct implementing `Rule`.
2. Register it in `rules::all()` (order matters — see the current ordering).
3. Unit-test it and confirm idempotence.

## Gotchas

- `PathBuf` has no `Display`; in `thiserror` messages use `{}` with `.path.display()`.
- Empty SQL params: `rusqlite::params![]`.
- bearcli write-back drives the live Bear app, one process per changed note —
  fine for a one-shot run, potentially slow for thousands of notes.
