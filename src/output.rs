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

fn terminal_width() -> usize {
    use terminal_size::{terminal_size, Width};
    terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(120)
}
fn truncate(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        let cut: String = chars[..max.saturating_sub(1)].iter().collect();
        format!("{}…", cut)
    }
}

fn print_table(headers: &[String], rows: &[Vec<String>], col_caps: &[usize]) {
    if headers.is_empty() {
        return;
    }
    let num_cols = headers.len();
    let sep = 2usize;
    let term_w = terminal_width();

    // Compute content-driven widths, capped per column
    let mut col_widths: Vec<usize> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| h.len().min(col_caps[i]))
        .collect();

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            let w = cell.len().min(col_caps[i]);
            if w > col_widths[i] {
                col_widths[i] = w;
            }
        }
    }

    // Last column (PATH) gets whatever terminal space is left
    let fixed: usize = col_widths[..num_cols - 1].iter().sum::<usize>() + sep * (num_cols - 1);
    let path_w = term_w.saturating_sub(fixed + sep).max(20);
    col_widths[num_cols - 1] = path_w;

    // Header row
    for (i, h) in headers.iter().enumerate() {
        if i == num_cols - 1 {
            print!("{}", h);
        } else {
            print!("{:<width$}{}", h, " ".repeat(sep), width = col_widths[i]);
        }
    }
    println!();

    // Separator line
    for (i, &w) in col_widths.iter().enumerate() {
        let total = if i == num_cols - 1 { w } else { w + sep };
        print!("{}", "─".repeat(total));
    }
    println!();

    // Data rows
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            let truncated = truncate(cell, col_widths[i]);
            if i == num_cols - 1 {
                print!("{}", truncated);
            } else {
                print!(
                    "{:<width$}{}",
                    truncated,
                    " ".repeat(sep),
                    width = col_widths[i]
                );
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
            if let Ok(s) = serde_json::to_string_pretty(&json_entries) {
                println!("{}", s);
            }
        }
        OutputMode::Table => {
            let headers = vec![
                "ID".to_string(),
                "FILENAME".to_string(),
                "SIZE".to_string(),
                "REPO".to_string(),
                "CATEGORY".to_string(),
                "PATH".to_string(),
            ];
            let caps = [6, 38, 9, 15, 32, usize::MAX];
            let rows: Vec<Vec<String>> = entries
                .iter()
                .map(|e| {
                    vec![
                        e.id.unwrap_or(0).to_string(),
                        e.filename.clone(),
                        format_size(e.size_bytes),
                        e.source_repo.clone().unwrap_or_else(|| ".".to_string()),
                        e.category.clone().unwrap_or_else(|| ".".to_string()),
                        e.path.clone(),
                    ]
                })
                .collect();
            print_table(&headers, &rows, &caps);
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
            if let Ok(s) = serde_json::to_string_pretty(&json_entries) {
                println!("{}", s);
            }
        }
        OutputMode::Table => {
            let headers = vec![
                "SCORE".to_string(),
                "FILENAME".to_string(),
                "REPO".to_string(),
                "PATH".to_string(),
            ];
            let caps = [6, 38, 15, usize::MAX];
            let rows: Vec<Vec<String>> = results
                .iter()
                .map(|r| {
                    vec![
                        r.score.to_string(),
                        r.filename.clone(),
                        r.repo.clone(),
                        r.path.clone(),
                    ]
                })
                .collect();
            print_table(&headers, &rows, &caps);
        }
    }
}
