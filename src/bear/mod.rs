//! Read-only access to the Bear SQLite database.
//!
//! Bear stores each note's canonical Markdown in `ZSFNOTE.ZTEXT`; titles,
//! backlinks, and the various `ZHAS…` flags are derived caches. This reader
//! only ever reads. Fixes are applied through Bear's own CLI (a later concern),
//! so sync state and those caches stay consistent.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use rusqlite::{Connection, OpenFlags};

/// One Bear note selected for linting.
pub struct Note {
    /// Bear's stable unique identifier for the note.
    pub identifier: String,
    /// The note's title (its first line), for display.
    pub title: String,
    /// The note's canonical Markdown.
    pub text: String,
}

/// What to lint.
pub enum Selector {
    /// A single note, by unique identifier.
    Note(String),
    /// Every note under a tag, including nested tags.
    Tag(String),
    /// The whole database.
    All,
}

/// A read-only handle to a Bear database.
pub struct BearDatabase {
    connection: Connection,
}

impl BearDatabase {
    /// The default database path inside Bear's group container.
    pub fn default_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        Some(home.join(
            "Library/Group Containers/9K33E3U3T4.net.shinyfrog.bear/Application Data/database.sqlite",
        ))
    }

    /// Open the database at `path` read-only. Bear may hold the database open
    /// concurrently; read-only access is safe alongside it.
    pub fn open(path: &Path) -> Result<Self, BearError> {
        let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|source| BearError::Open {
                path: path.to_path_buf(),
                source,
            })?;
        Ok(Self { connection })
    }

    /// Resolve `selector` into the notes it names.
    pub fn select(&self, selector: &Selector) -> Result<Vec<Note>, BearError> {
        match selector {
            Selector::Note(identifier) => self.note(identifier),
            Selector::Tag(tag) => self.notes_for_tag(tag),
            Selector::All => self.all_notes(),
        }
    }

    /// How many notes are locked (encrypted) and therefore skipped — Bear's CLI
    /// cannot read encrypted content, so there is no way to format them.
    pub fn locked_note_count(&self) -> Result<usize, BearError> {
        self.connection
            .query_row(
                "SELECT COUNT(*) FROM ZSFNOTE \
                 WHERE ZENCRYPTED = 1 AND ZTRASHED = 0 AND ZPERMANENTLYDELETED = 0",
                rusqlite::params![],
                |row| row.get(0),
            )
            .map_err(BearError::Query)
    }

    fn all_notes(&self) -> Result<Vec<Note>, BearError> {
        self.query(
            &format!("SELECT {NOTE_COLUMNS} FROM ZSFNOTE WHERE {LIVE_NOTE} ORDER BY {RECENT}"),
            rusqlite::params![],
        )
    }

    fn note(&self, identifier: &str) -> Result<Vec<Note>, BearError> {
        self.query(
            &format!(
                "SELECT {NOTE_COLUMNS} FROM ZSFNOTE \
                 WHERE {LIVE_NOTE} AND ZSFNOTE.ZUNIQUEIDENTIFIER = ?1"
            ),
            rusqlite::params![identifier],
        )
    }

    fn notes_for_tag(&self, tag: &str) -> Result<Vec<Note>, BearError> {
        self.query(
            &format!(
                "SELECT {NOTE_COLUMNS} FROM ZSFNOTE \
                 JOIN Z_5TAGS ON Z_5TAGS.Z_5NOTES = ZSFNOTE.Z_PK \
                 JOIN ZSFNOTETAG ON ZSFNOTETAG.Z_PK = Z_5TAGS.Z_13TAGS \
                 WHERE {LIVE_NOTE} \
                 AND (ZSFNOTETAG.ZTITLE = ?1 OR ZSFNOTETAG.ZTITLE LIKE ?1 || '/%') \
                 GROUP BY ZSFNOTE.Z_PK ORDER BY {RECENT}"
            ),
            rusqlite::params![tag],
        )
    }

    fn query(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<Vec<Note>, BearError> {
        let mut statement = self.connection.prepare(sql).map_err(BearError::Query)?;
        let rows = statement
            .query_map(params, |row| {
                Ok(Note {
                    identifier: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                    title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    text: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                })
            })
            .map_err(BearError::Query)?;
        let notes = rows
            .collect::<Result<Vec<Note>, rusqlite::Error>>()
            .map_err(BearError::Query)?;
        Ok(notes)
    }
}

/// The path to Bear's bundled CLI, if it is installed.
pub fn bearcli_path() -> Option<PathBuf> {
    let path = PathBuf::from("/Applications/Bear.app/Contents/MacOS/bearcli");
    path.is_file().then_some(path)
}

/// Overwrite a note's entire content through Bear's CLI, preserving the
/// modification date. Content is written on stdin so it lands verbatim, and the
/// attachment-removal safety gate is left on (no `--force`) — a write that would
/// drop an attachment is rejected rather than silently destructive.
pub fn overwrite_note(bearcli: &Path, id: &str, content: &str) -> Result<(), BearError> {
    let launch = |source| BearError::Bearcli {
        id: id.to_string(),
        source,
    };
    let mut child = Command::new(bearcli)
        .args(["overwrite", id, "--no-update-modified"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(launch)?;
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(content.as_bytes())
        .map_err(launch)?;
    let output = child.wait_with_output().map_err(launch)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(BearError::Overwrite {
            id: id.to_string(),
            message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        })
    }
}

// Columns are qualified with `ZSFNOTE.` throughout: the tag query joins
// `ZSFNOTETAG`, which shares several column names (ZUNIQUEIDENTIFIER,
// ZENCRYPTED, ZMODIFICATIONDATE, …), so unqualified references are ambiguous.

/// The note columns every query selects, in [`Note`] field order.
const NOTE_COLUMNS: &str = "ZSFNOTE.ZUNIQUEIDENTIFIER, ZSFNOTE.ZTITLE, ZSFNOTE.ZTEXT";

/// Predicate matching notes worth formatting: present text, not trashed, not
/// permanently deleted, not encrypted (whose text is unreadable anyway).
const LIVE_NOTE: &str = "ZSFNOTE.ZTEXT IS NOT NULL AND ZSFNOTE.ZTRASHED = 0 \
    AND ZSFNOTE.ZPERMANENTLYDELETED = 0 AND ZSFNOTE.ZENCRYPTED = 0";

/// Most-recently-modified first.
const RECENT: &str = "ZSFNOTE.ZMODIFICATIONDATE DESC";

/// Errors raised while reading the Bear database.
#[derive(Debug, thiserror::Error)]
pub enum BearError {
    /// The database file could not be opened.
    #[error("could not open the Bear database at {}", .path.display())]
    Open {
        /// The path we tried to open.
        path: PathBuf,
        /// The underlying SQLite error.
        #[source]
        source: rusqlite::Error,
    },
    /// A query against the database failed.
    #[error("a Bear database query failed")]
    Query(#[source] rusqlite::Error),
    /// bearcli could not be launched.
    #[error("could not run bearcli for note {id}")]
    Bearcli {
        /// The note being written.
        id: String,
        /// The underlying process error.
        #[source]
        source: std::io::Error,
    },
    /// bearcli rejected the overwrite.
    #[error("bearcli rejected the write for note {id}: {message}")]
    Overwrite {
        /// The note being written.
        id: String,
        /// bearcli's stderr message.
        message: String,
    },
}
