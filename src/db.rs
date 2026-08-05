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
}

pub fn init(path: &Path) -> Result<Connection, WlError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(path)?;

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

    Ok(conn)
}

pub fn upsert(conn: &Connection, entry: &WordlistEntry) -> Result<(), WlError> {
    conn.execute(
        "INSERT OR REPLACE INTO wordlists (
            filename, stem, path, extension, size_bytes, source_repo, category, compressed, line_count, mtime, last_indexed
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
        ),
    )?;
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
    })
}

pub fn get_by_name(conn: &Connection, name: &str) -> Result<Vec<WordlistEntry>, WlError> {
    let mut stmt = conn.prepare(
        "SELECT id, filename, stem, path, extension, size_bytes, source_repo, category, compressed, line_count, mtime, last_indexed
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
        "SELECT id, filename, stem, path, extension, size_bytes, source_repo, category, compressed, line_count, mtime, last_indexed
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

pub fn get_mtime(conn: &Connection, path: &str) -> Result<Option<i64>, WlError> {
    let mut stmt = conn.prepare("SELECT mtime FROM wordlists WHERE path = ?1")?;
    let mut rows = stmt.query([path])?;
    if let Some(row) = rows.next()? {
        let mtime: i64 = row.get(0)?;
        Ok(Some(mtime))
    } else {
        Ok(None)
    }
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
