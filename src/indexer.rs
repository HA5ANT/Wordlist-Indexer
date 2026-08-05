use crate::db::{self, WordlistEntry};
use crate::error::WlError;
use chrono::Utc;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn get_valid_extension(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "txt" | "lst" | "list" | "dict" | "wordlist" | "gz" | "zip" | "bz2" | "xz" => Some(ext),
        _ => None,
    }
}

fn is_compressed(ext: &str) -> bool {
    matches!(ext, "gz" | "zip" | "tar" | "bz2" | "xz")
}

fn count_lines_fast(path: &Path) -> Option<i64> {
    use std::fs::File;
    use std::io::Read;
    let mut file = File::open(path).ok()?;
    let mut buffer = [0; 64 * 1024];
    let mut count = 0;
    loop {
        let n = file.read(&mut buffer).ok()?;
        if n == 0 {
            break;
        }
        count += buffer[..n].iter().filter(|&&b| b == b'\n').count() as i64;
    }
    Some(count)
}

pub fn index_all(conn: &Connection, repos: &[PathBuf], quiet: bool) -> Result<(), WlError> {
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

            // Check if mtime is unchanged
            if let Ok(Some(existing_mtime)) = db::get_mtime(conn, &path_str) {
                if existing_mtime == mtime {
                    skipped_unchanged += 1;
                    continue;
                }
            }

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

    // Clean up files in DB that are no longer present in any configured repos
    db::delete_missing(conn, &all_indexed_paths)?;

    Ok(())
}
