# bear-formatter 🐻

A cute little formatter for your Bear notes.

**bear-formatter tidies the Markdown in your [Bear](https://bear.app) notes** —
consistent bullets, spacing, headings, and more. It's a pure formatter (think
gofmt, not a linter): notes in, canonical notes out. It reads straight from
Bear's database and — soon — writes fixes back through Bear's own CLI so your
notes stay in sync.

> ⚠️ **Early days.** The engine, CLI, and read-only Bear reader are in place with
> one demonstrator rule (`final-newline`). The real rule catalog, write-back, and
> the Homebrew formula are next.

## Install

Homebrew (coming):

```bash
brew install chasefinch/tap/bear-formatter
```

From source:

```bash
git clone --recurse-submodules https://github.com/chasefinch/bear-formatter.git
cd bear-formatter
make build   # binary at target/release/bear-formatter
```

## Usage

Format a Markdown string — the live path today:

```bash
bear-formatter --code "# Title
- item"
```

Format notes from your Bear database — a single note, a tag (including nested
tags), or everything:

```bash
bear-formatter --all                 # every note
bear-formatter --tag Recipes         # #Recipes and #Recipes/*
bear-formatter --note <UUID>         # one note
```

Until write-back lands, database targets report which notes *would* change.
Point at a copy while experimenting:

```bash
bear-formatter --all --database /path/to/a/copy.sqlite
```

## How it reads Bear

bear-formatter opens Bear's SQLite database **read-only** and treats
`ZSFNOTE.ZTEXT` as the source of truth. Trashed, deleted, and encrypted notes
are skipped. It never writes to the database directly — fixes will be applied
through Bear's CLI so sync state stays intact.

## Development

```bash
make        # format → lint → check → test (Rust source quality)
```

Engineering conventions live in the [`axioms`](axioms) submodule.

## License

MIT — see [LICENSE](LICENSE).
