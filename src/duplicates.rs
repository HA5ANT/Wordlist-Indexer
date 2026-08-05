use crate::db;
use crate::error::WlError;
use crate::output::OutputMode;
use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DupMode {
    Name,
    Size,
    Hash,
}

#[derive(Serialize)]
struct DuplicateGroup {
    key: String,
    paths: Vec<String>,
}

pub fn run_duplicates(
    conn: &Connection,
    dup_mode: DupMode,
    mode: OutputMode,
) -> Result<(), WlError> {
    let all = db::get_all(conn)?;
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();

    for entry in all {
        let key = match dup_mode {
            DupMode::Name => entry.filename.clone(),
            DupMode::Size => entry.size_bytes.to_string(),
            DupMode::Hash => entry.sha256.clone().unwrap_or_default(),
        };
        if !key.is_empty() {
            groups.entry(key).or_default().push(entry.path);
        }
    }

    let mut duplicate_groups: Vec<DuplicateGroup> = groups
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(key, mut paths)| {
            paths.sort();
            DuplicateGroup { key, paths }
        })
        .collect();

    // Sort groups by key
    duplicate_groups.sort_by(|a, b| a.key.cmp(&b.key));

    match mode {
        OutputMode::Json => {
            println!("{}", serde_json::to_string(&duplicate_groups).unwrap());
        }
        OutputMode::Table => {
            if duplicate_groups.is_empty() {
                return Ok(());
            }
            let mut col_widths = [13, 4];
            for group in &duplicate_groups {
                if group.key.len() > col_widths[0] {
                    col_widths[0] = group.key.len();
                }
                for p in &group.paths {
                    if p.len() > col_widths[1] {
                        col_widths[1] = p.len();
                    }
                }
            }

            // Print header
            println!("{:<width$}  PATH", "DUPLICATE KEY", width = col_widths[0]);
            println!(
                "{}  {}",
                "-".repeat(col_widths[0]),
                "-".repeat(col_widths[1])
            );

            for group in &duplicate_groups {
                for p in &group.paths {
                    println!("{:<width$}  {}", group.key, p, width = col_widths[0]);
                }
            }
        }
        OutputMode::Plain => {
            for group in &duplicate_groups {
                println!("{}", group.key);
                for p in &group.paths {
                    println!("    {}", p);
                }
                println!();
                println!("-----------------------------------");
            }
        }
    }

    Ok(())
}
