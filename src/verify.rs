use crate::db;
use crate::error::WlError;
use rusqlite::Connection;
use std::fs;

pub fn run_verify(conn: &Connection) -> Result<(), WlError> {
    // 1. Check database readability
    let db_readable = db::get_all(conn).is_ok();
    if db_readable {
        println!("✓ Database readable");
    } else {
        println!("✗ Database unreadable");
    }
    println!();

    // 2. SQLite integrity OK
    let mut integrity_ok = false;
    if let Ok(mut stmt) = conn.prepare("PRAGMA integrity_check") {
        if let Ok(mut rows) = stmt.query([]) {
            if let Ok(Some(row)) = rows.next() {
                let status: String = row.get(0)?;
                if status == "ok" {
                    integrity_ok = true;
                }
            }
        }
    }
    if integrity_ok {
        println!("✓ SQLite integrity OK");
    } else {
        println!("✗ SQLite integrity check FAILED");
    }
    println!();

    // Load entries to verify files
    let entries = db::get_all(conn)?;
    let total_entries = entries.len();

    let mut missing_files = 0;
    let mut broken_symlinks = 0;
    let mut unreadable_files = 0;

    for entry in &entries {
        let sym_meta = fs::symlink_metadata(&entry.path);
        match sym_meta {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    // Check if target is broken
                    if fs::metadata(&entry.path).is_err() {
                        broken_symlinks += 1;
                    }
                } else if fs::File::open(&entry.path).is_err() {
                    unreadable_files += 1;
                }
            }
            Err(_) => {
                missing_files += 1;
            }
        }
    }

    println!("{} entries checked", total_entries);
    println!();

    let existing_files = total_entries - missing_files;
    println!("✓ {} files exist", existing_files);
    println!();

    if missing_files > 0 {
        println!("⚠ {} missing files", missing_files);
    } else {
        println!("✓ 0 missing files");
    }
    println!();

    if unreadable_files > 0 {
        println!("⚠ {} unreadable files", unreadable_files);
    } else {
        println!("✓ 0 unreadable files");
    }
    println!();

    // 3. Duplicate DB entries check
    let total_rows: i64 = conn.query_row("SELECT COUNT(*) FROM wordlists", [], |r| r.get(0))?;
    let distinct_paths: i64 =
        conn.query_row("SELECT COUNT(DISTINCT path) FROM wordlists", [], |r| {
            r.get(0)
        })?;
    let duplicate_db_entries = total_rows - distinct_paths;

    if duplicate_db_entries > 0 {
        println!("⚠ {} duplicate DB entries", duplicate_db_entries);
    } else {
        println!("✓ 0 duplicate DB entries");
    }
    println!();

    if broken_symlinks > 0 {
        println!("⚠ {} broken symlinks", broken_symlinks);
    } else {
        println!("✓ 0 broken symlinks");
    }
    println!();

    // Check if verification passes overall
    let pass = db_readable && integrity_ok && duplicate_db_entries == 0;
    if pass {
        println!("PASS");
    } else {
        println!("FAIL");
    }

    Ok(())
}
