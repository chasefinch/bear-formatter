# Rules

bear-formatter is a **formatter**, not a linter: notes in, canonical notes out.
Every rule is a total, idempotent transformation, applied in the order below.
Code (fenced, indented, and inline) is left untouched by every rule.

| # | Rule | What it does |
|---|------|--------------|
| 1 | `line-endings` | CRLF / lone CR → LF. |
| 2 | `typography` | Straight quotes/apostrophes → curly; `...` → `…`; spaced `. . .` (three or more dots) → `…`. |
| 3 | `whitespace` | Collapse runs of spaces, drop spaces before punctuation, empty whitespace-only lines, keep two-space hard breaks. |
| 4 | `horizontal-rules` | Divider lines → `---`: Markdown `-`/`_`/`*` runs (`-----`, `- - -`, `* * *`; three or more marks), and Unicode dash lines pasted as dividers (en/em/figure dashes, horizontal bar, `⸺`, `⸻`; even a single one), optionally spaced or surrounded by whitespace. Lines inside code/quotes/tables untouched. |
| 5 | `headings` | One space after the `#`s, no leading indent, trailing punctuation trimmed. Casing untouched. |
| 6 | `list-markers` | Bullets → `-`; a marker followed by a tab or spaces (e.g. pasted `•\t`) gets exactly one space; a bare `.` followed by a tab is a pasted marker too (a numbered item that lost its number — `. prose` is left alone); drop empty items; collapse duplicated markers. Ordered numbers untouched (no renumbering). |
| 7 | `footnotes` | Renumber by first-reference order; move definitions to the bottom. |
| 8 | `tags` | Tag-led lines moved under the note's opening heading, or to the bottom when it doesn't open with a heading (so tags never become the title/preview): bare tags merged, deduped, and sorted onto one line; a tag with trailing text (e.g. a meeting date) kept on its own line; multi-tag lines split per tag; redundant closing `#` stripped. Tags mid-prose stay put. |
| 9 | `title-case` | The note's title (a first-line heading) is title-cased: small words lowercased in the middle, first and last always capitalized, words with an interior capital (URL, iPhone) left alone, nothing else lowercased. Other headings keep their casing. |
| 10 | `heading-levels` | Biggest heading promoted to H1 (or H2 when the note opens with prose — Bear treats line 1 as the title); no heading jumps more than one level deeper than the previous. Multiple/zero H1s are fine. |
| 11 | `tables` | A flush-left line containing a tab (and not already a heading/list/quote/tag/footnote/pipe line) becomes a table row — tabs → cell dividers, pipes in cells escaped — followed by the minimal `\| - \| - \|` separator. A head-only table separated from a preceding table by only whitespace merges into it as a body row (type `a<tab>b`, blanks, `c<tab>d` → head + body); tables that already have bodies never merge. Tables re-emit canonically: cells trimmed, separator minimized (alignment kept), ragged rows padded with empty cells to the widest row. Pipe blocks without a separator row are left alone. |
| 12 | `layout` | One blank line around every block; tag-led lines hug the heading; a single newline between prose lines becomes a paragraph break (Bear model); blockquote paragraphs split with empty `>` lines (`> \|` keeps them together); consecutive wikilink-only lines are a contiguous table of contents; list indent → tabs; no blanks between same-kind items (bullets and todos are one list; a numbered list is separate and gets a blank); no blank between an item and its nested sub-list (a sub-list is part of its item, not a paragraph) — unless a continuation paragraph sits between them, which puts a blank before the sub-list too; a multi-paragraph item (continuation prose after a blank) gets a trailing blank symmetric with the one before its prose; blanks around root lists; whole-line bold labels (`**Label:**`) are their own block; a trailing horizontal rule is stripped; no leading/trailing blanks. |
| 13 | `final-newline` | Exactly one trailing newline. |

## Design notes

- **Tags parse from text** (no database): a tag is `#` + non-space characters,
  optionally closed with `#`. This runs identically against files or the DB.
- **Newlines are paragraph breaks.** Bear wraps text automatically, so a manual
  newline always means a new paragraph; consecutive prose lines are split with a
  blank line (an explicit two-space hard break keeps lines together).
- **Keep lines together with a leading `|`.** A line starting with `|` is a
  table row, and consecutive rows stay contiguous — so a leading pipe is a simple
  "don't split these into paragraphs" marker (address blocks, verse, etc.).
  A blank between pipe blocks separates them — two tables stay two tables;
  deliberate merging is the `tables` rule's job — and a blank is enforced after
  each block. (Bear renders a lone `| text` line with a visible pipe unless it's
  a real table with a `| --- |` separator row.)
- **Idempotence** is the contract and is checked in `tests/integration.rs`; it
  also held across 60 real notes during development.

## Known v1 gaps (to revisit after testing)

- **Spaced tags** (`#a b#`, closed with `#`) are left untouched rather than
  reformatted — only simple `#tag` forms are gathered/sorted/split. Their
  redundant-`#` stripping is therefore skipped (which is safe — the `#` is kept).
- **List continuation lines** (indented text under an item) are re-indented with
  tabs at one level past the item, but their original alignment is not otherwise
  preserved.
- **Blank lines inside lists** are emitted empty, not indented to the list level.
- **Footnotes**: only single-line definitions are moved; multi-line definitions
  stay where they are.
