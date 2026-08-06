use crate::error::WlError;
use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordlistEntry {
    pub id: Option<i64>,
    pub filename: String,
    pub stem: String,
    pub path: String,
    pub extension: Option<String>,
    pub size_bytes: i64,
    pub source_repo: Option<String>,
    pub category: Option<String>,
    pub compressed: bool,
    pub line_count: Option<i64>,
    pub mtime: i64,
    pub last_indexed: i64,
    pub sha256: Option<String>,
}

pub fn init(path: &Path) -> Result<Connection, WlError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if path.exists() {
        fs::copy(path, path.with_extension("db.bak"))?;
    }

    let conn = Connection::open(path)?;

    // Initialize migration table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        );",
        [],
    )?;

    // Get current version
    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Run migrations
    if current_version < 1 {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS wordlists (
              id           INTEGER PRIMARY KEY,
              filename     TEXT NOT NULL,
              stem         TEXT NOT NULL,
              path         TEXT NOT NULL UNIQUE,
              extension    TEXT,
              size_bytes   INTEGER,
              source_repo  TEXT,
              category     TEXT,
              compressed   BOOLEAN DEFAULT 0,
              line_count   INTEGER,
              mtime        INTEGER,
              last_indexed INTEGER
            );",
            [],
        )?;

        conn.execute("INSERT INTO schema_version (version) VALUES (1)", [])?;
    }

    if current_version < 2 {
        let _ = conn.execute("ALTER TABLE wordlists ADD COLUMN sha256 TEXT", []);
        conn.execute("INSERT INTO schema_version (version) VALUES (2)", [])?;
    }

    if current_version < 3 {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY,
                name TEXT UNIQUE NOT NULL
            );",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS wordlist_tags (
                wordlist_id INTEGER REFERENCES wordlists(id) ON DELETE CASCADE,
                tag_id INTEGER REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY (wordlist_id, tag_id)
            );",
            [],
        )?;
        conn.execute("INSERT INTO schema_version (version) VALUES (3)", [])?;
    }

    if current_version < 4 {
        conn.execute(
            "ALTER TABLE wordlist_tags ADD COLUMN is_manual BOOLEAN DEFAULT 0",
            [],
        )?;
        conn.execute("INSERT INTO schema_version (version) VALUES (4)", [])?;
    }

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_filename ON wordlists(filename);",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stem     ON wordlists(stem);",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_repo     ON wordlists(source_repo);",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_category ON wordlists(category);",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_extension ON wordlists(extension);",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sha256 ON wordlists(sha256);",
        [],
    )?;

    Ok(conn)
}

pub fn upsert(conn: &Connection, entry: &WordlistEntry) -> Result<i64, WlError> {
    conn.execute(
        "INSERT OR REPLACE INTO wordlists (
            filename, stem, path, extension, size_bytes, source_repo, category, compressed, line_count, mtime, last_indexed, sha256
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        (
            &entry.filename,
            &entry.stem,
            &entry.path,
            &entry.extension,
            entry.size_bytes,
            &entry.source_repo,
            &entry.category,
            entry.compressed,
            entry.line_count,
            entry.mtime,
            entry.last_indexed,
            &entry.sha256,
        ),
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn set_tags_for_wordlist(
    conn: &Connection,
    wordlist_id: i64,
    tags: &[String],
    is_manual: bool,
) -> Result<(), WlError> {
    for tag in tags {
        conn.execute("INSERT OR IGNORE INTO tags (name) VALUES (?1)", [tag])?;
    }

    if !is_manual {
        conn.execute(
            "DELETE FROM wordlist_tags WHERE wordlist_id = ?1 AND is_manual = 0",
            [wordlist_id],
        )?;
    }

    for tag in tags {
        let tag_id: i64 = conn.query_row("SELECT id FROM tags WHERE name = ?1", [tag], |row| {
            row.get(0)
        })?;
        conn.execute(
            "INSERT OR REPLACE INTO wordlist_tags (wordlist_id, tag_id, is_manual) VALUES (?1, ?2, ?3)", 
            (wordlist_id, tag_id, if is_manual { 1 } else { 0 })
        )?;
    }
    Ok(())
}

pub fn get_tags_for_wordlist(conn: &Connection, wordlist_id: i64) -> Result<Vec<String>, WlError> {
    let mut stmt = conn.prepare("SELECT t.name FROM tags t JOIN wordlist_tags wt ON t.id = wt.tag_id WHERE wt.wordlist_id = ?1")?;
    let rows = stmt.query_map([wordlist_id], |row| row.get(0))?;
    let mut tags = Vec::new();
    for row in rows {
        tags.push(row?);
    }
    Ok(tags)
}

pub fn add_tag_to_wordlist(conn: &Connection, wordlist_id: i64, tag: &str) -> Result<(), WlError> {
    conn.execute("INSERT OR IGNORE INTO tags (name) VALUES (?1)", [tag])?;
    let tag_id: i64 = conn.query_row("SELECT id FROM tags WHERE name = ?1", [tag], |row| row.get(0))?;
    conn.execute(
        "INSERT OR REPLACE INTO wordlist_tags (wordlist_id, tag_id, is_manual) VALUES (?1, ?2, 1)", 
        (wordlist_id, tag_id)
    )?;
    Ok(())
}

pub fn remove_tag_from_wordlist(conn: &Connection, wordlist_id: i64, tag: &str) -> Result<(), WlError> {
    let tag_id: i64 = conn.query_row("SELECT id FROM tags WHERE name = ?1", [tag], |row| row.get(0))?;
    conn.execute("DELETE FROM wordlist_tags WHERE wordlist_id = ?1 AND tag_id = ?2", (wordlist_id, tag_id))?;
    Ok(())
}

fn row_to_entry(row: &rusqlite::Row) -> SqliteResult<WordlistEntry> {
    Ok(WordlistEntry {
        id: Some(row.get(0)?),
        filename: row.get(1)?,
        stem: row.get(2)?,
        path: row.get(3)?,
        extension: row.get(4)?,
        size_bytes: row.get(5)?,
        source_repo: row.get(6)?,
        category: row.get(7)?,
        compressed: row.get(8)?,
        line_count: row.get(9)?,
        mtime: row.get(10)?,
        last_indexed: row.get(11)?,
        sha256: row.get(12)?,
    })
}

pub fn get_by_name(conn: &Connection, name: &str) -> Result<Vec<WordlistEntry>, WlError> {
    let mut stmt = conn.prepare(
        "SELECT id, filename, stem, path, extension, size_bytes, source_repo, category, compressed, line_count, mtime, last_indexed, sha256
         FROM wordlists
         WHERE LOWER(filename) = LOWER(?1) OR LOWER(stem) = LOWER(?1)
         ORDER BY path"
    )?;

    let rows = stmt.query_map([name], row_to_entry)?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn get_all(conn: &Connection) -> Result<Vec<WordlistEntry>, WlError> {
    let mut stmt = conn.prepare(
        "SELECT id, filename, stem, path, extension, size_bytes, source_repo, category, compressed, line_count, mtime, last_indexed, sha256
         FROM wordlists
         ORDER BY path"
    )?;
    let rows = stmt.query_map([], row_to_entry)?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn get_entry_by_path(conn: &Connection, path: &str) -> Result<Option<WordlistEntry>, WlError> {
    let mut stmt = conn.prepare(
        "SELECT id, filename, stem, path, extension, size_bytes, source_repo, category, compressed, line_count, mtime, last_indexed, sha256
         FROM wordlists
         WHERE path = ?1"
    )?;
    let mut rows = stmt.query_map([path], row_to_entry)?;
    if let Some(row) = rows.next() {
        Ok(Some(row?))
    } else {
        Ok(None)
    }
}

pub fn delete_entry(conn: &Connection, path: &str) -> Result<(), WlError> {
    conn.execute("DELETE FROM wordlists WHERE path = ?1", [path])?;
    Ok(())
}

pub fn delete_missing(conn: &Connection, existing_paths: &[String]) -> Result<(), WlError> {
    if existing_paths.is_empty() {
        conn.execute("DELETE FROM wordlists", [])?;
        return Ok(());
    }

    conn.execute(
        "CREATE TEMPORARY TABLE IF NOT EXISTS temp_existing_paths (path TEXT UNIQUE)",
        [],
    )?;
    conn.execute("DELETE FROM temp_existing_paths", [])?;

    let mut stmt = conn.prepare("INSERT INTO temp_existing_paths (path) VALUES (?1)")?;
    for p in existing_paths {
        stmt.execute([p])?;
    }

    conn.execute(
        "DELETE FROM wordlists WHERE path NOT IN (SELECT path FROM temp_existing_paths)",
        [],
    )?;

    Ok(())
}
