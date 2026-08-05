use crate::db::{self, WordlistEntry};
use crate::error::WlError;
use crate::indexer::{
    compute_sha256, count_lines_fast, get_valid_extension, is_compressed, is_hidden,
};
use chrono::Utc;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn update_incremental(
    conn: &Connection,
    repos: &[PathBuf],
    quiet: bool,
) -> Result<(), WlError> {
    let mut all_indexed_paths = Vec::new();

    for repo in repos {
        if !repo.exists() {
            if !quiet {
                eprintln!("Repository path does not exist: {:?}", repo);
            }
            continue;
        }

        if !quiet {
            eprintln!("Scanning {}...", repo.display());
        }

        let source_repo = repo
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown".to_string());

        let mut added = 0;
        let mut skipped_unchanged = 0;
        let mut skipped_large = 0;

        let walker = WalkDir::new(repo)
            .into_iter()
            .filter_entry(|e| !is_hidden(e));

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = match get_valid_extension(path) {
                Some(e) => e,
                None => continue,
            };

            let metadata = match fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let size_bytes = metadata.len();
            if size_bytes > 500 * 1024 * 1024 {
                skipped_large += 1;
                continue;
            }

            let mtime = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let path_str = path.to_string_lossy().into_owned();
            all_indexed_paths.push(path_str.clone());

            // Check if mtime and size match in DB
            if let Ok(Some(existing)) = db::get_entry_by_path(conn, &path_str) {
                if existing.mtime == mtime
                    && existing.size_bytes == size_bytes as i64
                    && existing.sha256.is_some()
                {
                    skipped_unchanged += 1;
                    continue;
                }
            }

            // Otherwise, read and compute hash
            let sha256 = compute_sha256(path).ok();

            let filename = path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();

            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();

            let relative = path.strip_prefix(repo).unwrap_or(path);
            let category = relative
                .parent()
                .map(|p| p.to_string_lossy().into_owned())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| ".".to_string());

            let compressed = is_compressed(&ext);

            let line_count = if compressed {
                None
            } else {
                count_lines_fast(path)
            };

            let db_entry = WordlistEntry {
                id: None,
                filename,
                stem,
                path: path_str,
                extension: Some(ext),
                size_bytes: size_bytes as i64,
                source_repo: Some(source_repo.clone()),
                category: Some(category),
                compressed,
                line_count,
                mtime,
                last_indexed: Utc::now().timestamp(),
                sha256,
            };

            db::upsert(conn, &db_entry)?;
            added += 1;
        }

        if !quiet {
            eprintln!(
                "[+] {} added  |  {} skipped (unchanged)  |  {} skipped (too large)",
                added, skipped_unchanged, skipped_large
            );
        }
    }

    db::delete_missing(conn, &all_indexed_paths)?;

    Ok(())
}
