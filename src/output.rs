use crate::db::WordlistEntry;
use crate::search::FuzzyResult;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Plain,
    Json,
    Table,
}

#[derive(Serialize)]
struct JsonEntry {
    filename: String,
    path: String,
    repo: String,
    size_bytes: i64,
}

#[derive(Serialize)]
struct JsonFuzzyEntry {
    score: i64,
    filename: String,
    repo: String,
    path: String,
}

fn format_size(bytes: i64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

fn print_table(headers: &[String], rows: &[Vec<String>]) {
    if headers.is_empty() {
        return;
    }
    let num_cols = headers.len();
    let mut col_widths = vec![0; num_cols];

    for (i, h) in headers.iter().enumerate() {
        col_widths[i] = h.len();
    }

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if cell.len() > col_widths[i] {
                col_widths[i] = cell.len();
            }
        }
    }

    for (i, h) in headers.iter().enumerate() {
        if i == num_cols - 1 {
            print!("{}", h);
        } else {
            print!("{:<width$}  ", h, width = col_widths[i]);
        }
    }
    println!();

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i == num_cols - 1 {
                print!("{}", cell);
            } else {
                print!("{:<width$}  ", cell, width = col_widths[i]);
            }
        }
        println!();
    }
}

pub fn render_entries(entries: &[WordlistEntry], mode: OutputMode) {
    match mode {
        OutputMode::Plain => {
            for entry in entries {
                println!("{}", entry.path);
            }
        }
        OutputMode::Json => {
            let json_entries: Vec<JsonEntry> = entries
                .iter()
                .map(|e| JsonEntry {
                    filename: e.filename.clone(),
                    path: e.path.clone(),
                    repo: e
                        .source_repo
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                    size_bytes: e.size_bytes,
                })
                .collect();
            if let Ok(json_str) = serde_json::to_string(&json_entries) {
                println!("{}", json_str);
            }
        }
        OutputMode::Table => {
            let headers = vec![
                "FILENAME".to_string(),
                "SIZE".to_string(),
                "REPO".to_string(),
                "CATEGORY".to_string(),
                "PATH".to_string(),
            ];
            let mut rows = Vec::new();
            for entry in entries {
                rows.push(vec![
                    entry.filename.clone(),
                    format_size(entry.size_bytes),
                    entry.source_repo.clone().unwrap_or_else(|| ".".to_string()),
                    entry.category.clone().unwrap_or_else(|| ".".to_string()),
                    entry.path.clone(),
                ]);
            }
            print_table(&headers, &rows);
        }
    }
}

pub fn render_fuzzy(results: &[FuzzyResult], mode: OutputMode) {
    match mode {
        OutputMode::Plain => {
            for r in results {
                println!("{}", r.path);
            }
        }
        OutputMode::Json => {
            let json_entries: Vec<JsonFuzzyEntry> = results
                .iter()
                .map(|r| JsonFuzzyEntry {
                    score: r.score,
                    filename: r.filename.clone(),
                    repo: r.repo.clone(),
                    path: r.path.clone(),
                })
                .collect();
            if let Ok(json_str) = serde_json::to_string(&json_entries) {
                println!("{}", json_str);
            }
        }
        OutputMode::Table => {
            let headers = vec![
                "SCORE".to_string(),
                "FILENAME".to_string(),
                "REPO".to_string(),
                "PATH".to_string(),
            ];
            let mut rows = Vec::new();
            for r in results {
                rows.push(vec![
                    r.score.to_string(),
                    r.filename.clone(),
                    r.repo.clone(),
                    r.path.clone(),
                ]);
            }
            print_table(&headers, &rows);
        }
    }
}
