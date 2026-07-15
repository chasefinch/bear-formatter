# bear-formatter 🐻

[![CI](https://github.com/chasefinch/bear-formatter/actions/workflows/ci.yml/badge.svg)](https://github.com/chasefinch/bear-formatter/actions/workflows/ci.yml)
![Coverage](https://img.shields.io/badge/coverage-76%25-yellowgreen)

A formatter for your Bear notes. The command is **`bear-format`**.

**bear-format tidies the Markdown in your [Bear](https://bear.app) notes** —
consistent headings, bullets, spacing, tags, and more. It's a pure formatter
(think gofmt, not a linter): notes in, canonical notes out. It reads from Bear's
database and writes fixes back through Bear's own CLI, so sync state stays
intact.

## Install

Homebrew:

```bash
brew install chasefinch/tap/bear-formatter
```

From source:

```bash
git clone --recurse-submodules https://github.com/chasefinch/bear-formatter.git
cd bear-formatter
make build   # binary at target/release/bear-format
```

## Usage

`bear-format` edits **in place**. Point it at a Bear database (notes are
rewritten through Bear's CLI) and/or Markdown files or globs (rewritten on
disk):

```bash
bear-format ~/Library/Group\ Containers/9K33E3U3T4.net.shinyfrog.bear/Application\ Data/database.sqlite
bear-format "vault/**/*.md"
bear-format note.md another.md
```

Preview first with `--dry-run` — it writes nothing and lists what would change:

```bash
bear-format --dry-run "vault/**/*.md"
```

Format a Markdown string straight to stdout (touches nothing):

```bash
bear-format --code "# Title
- item"
```

## How it works

bear-format opens a Bear database **read-only** and treats `ZSFNOTE.ZTEXT` as
the source of truth (trashed, deleted, and encrypted notes are skipped). It
never writes the SQLite file directly — each changed note is written back with
`bearcli overwrite … --no-update-modified`, which keeps Bear's sync state and
derived caches consistent and preserves modification dates. bearcli's
attachment-removal safety gate is left on, so a formatting change that would drop
an attachment is rejected rather than applied.

The rules are documented in [`docs/rules.md`](docs/rules.md).

## Development

```bash
make        # format → lint → check → test (Rust source quality)
```

Engineering conventions live in the [`axioms`](axioms) submodule.

## License

MIT — see [LICENSE](LICENSE).
