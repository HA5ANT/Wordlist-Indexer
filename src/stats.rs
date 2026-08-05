use crate::error::WlError;
use crate::output::OutputMode;
use rusqlite::Connection;
use serde::Serialize;

#[derive(Serialize)]
struct GeneralStats {
    repositories_count: i64,
    wordlists_count: i64,
    total_disk_usage: i64,
    compressed_count: i64,
    text_count: i64,
    repositories: Vec<RepoStat>,
    categories: Vec<CategoryStat>,
}

#[derive(Serialize, Clone)]
struct RepoStat {
    repository: String,
    count: i64,
}

#[derive(Serialize, Clone)]
struct CategoryStat {
    category: String,
    count: i64,
}

#[derive(Serialize, Clone)]
struct ExtensionStat {
    extension: String,
    count: i64,
}

#[derive(Serialize, Clone)]
struct LargestStat {
    filename: String,
    size_bytes: i64,
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

    // Print divider
    for (i, w) in col_widths.iter().enumerate() {
        let dashes = "-".repeat(*w);
        if i == num_cols - 1 {
            print!("{}", dashes);
        } else {
            print!("{}  ", dashes);
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

pub fn run_stats(
    conn: &Connection,
    only_repo: bool,
    only_category: bool,
    only_largest: bool,
    only_extensions: bool,
    mode: OutputMode,
) -> Result<(), WlError> {
    if only_repo {
        let repos = query_repos(conn)?;
        match mode {
            OutputMode::Json => {
                println!("{}", serde_json::to_string(&repos).unwrap());
            }
            _ => {
                let headers = vec!["Repository".to_string(), "Files".to_string()];
                let rows: Vec<Vec<String>> = repos
                    .into_iter()
                    .map(|r| vec![r.repository, r.count.to_string()])
                    .collect();
                print_table(&headers, &rows);
            }
        }
        return Ok(());
    }

    if only_category {
        let cats = query_categories(conn)?;
        match mode {
            OutputMode::Json => {
                println!("{}", serde_json::to_string(&cats).unwrap());
            }
            _ => {
                let headers = vec!["Category".to_string(), "Files".to_string()];
                let rows: Vec<Vec<String>> = cats
                    .into_iter()
                    .map(|c| vec![c.category, c.count.to_string()])
                    .collect();
                print_table(&headers, &rows);
            }
        }
        return Ok(());
    }

    if only_largest {
        let largest = query_largest(conn)?;
        match mode {
            OutputMode::Json => {
                println!("{}", serde_json::to_string(&largest).unwrap());
            }
            _ => {
                let headers = vec![
                    "FILENAME".to_string(),
                    "SIZE".to_string(),
                    "REPOSITORY".to_string(),
                    "PATH".to_string(),
                ];
                let rows: Vec<Vec<String>> = largest
                    .into_iter()
                    .map(|l| vec![l.filename, format_size(l.size_bytes), l.repo, l.path])
                    .collect();
                print_table(&headers, &rows);
            }
        }
        return Ok(());
    }

    if only_extensions {
        let exts = query_extensions(conn)?;
        match mode {
            OutputMode::Json => {
                println!("{}", serde_json::to_string(&exts).unwrap());
            }
            _ => {
                let headers = vec!["Extension".to_string(), "Files".to_string()];
                let rows: Vec<Vec<String>> = exts
                    .into_iter()
                    .map(|e| vec![e.extension, e.count.to_string()])
                    .collect();
                print_table(&headers, &rows);
            }
        }
        return Ok(());
    }

    // Default overview stats
    let total_wordlists: i64 =
        conn.query_row("SELECT COUNT(*) FROM wordlists", [], |r| r.get(0))?;
    let total_disk: i64 = conn.query_row(
        "SELECT COALESCE(SUM(size_bytes), 0) FROM wordlists",
        [],
        |r| r.get(0),
    )?;
    let compressed: i64 = conn.query_row(
        "SELECT COUNT(*) FROM wordlists WHERE compressed = 1",
        [],
        |r| r.get(0),
    )?;
    let text: i64 = conn.query_row(
        "SELECT COUNT(*) FROM wordlists WHERE compressed = 0",
        [],
        |r| r.get(0),
    )?;
    let repos_count: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT source_repo) FROM wordlists",
        [],
        |r| r.get(0),
    )?;

    let repos = query_repos(conn)?;
    let cats = query_categories(conn)?;

    match mode {
        OutputMode::Json => {
            let stats = GeneralStats {
                repositories_count: repos_count,
                wordlists_count: total_wordlists,
                total_disk_usage: total_disk,
                compressed_count: compressed,
                text_count: text,
                repositories: repos,
                categories: cats,
            };
            println!("{}", serde_json::to_string(&stats).unwrap());
        }
        _ => {
            println!("Number of repositories:      {}", repos_count);
            println!("Number of indexed wordlists: {}", total_wordlists);
            println!("Total disk usage:            {}", format_size(total_disk));
            println!("Number of compressed files:  {}", compressed);
            println!("Number of indexed text files: {}", text);
            println!();

            println!("Repository Breakdown:");
            let repo_headers = vec!["Repository".to_string(), "Files".to_string()];
            let repo_rows: Vec<Vec<String>> = repos
                .into_iter()
                .map(|r| vec![r.repository, r.count.to_string()])
                .collect();
            print_table(&repo_headers, &repo_rows);
            println!();

            println!("Category Breakdown:");
            let cat_headers = vec!["Category".to_string(), "Files".to_string()];
            let cat_rows: Vec<Vec<String>> = cats
                .into_iter()
                .map(|c| vec![c.category, c.count.to_string()])
                .collect();
            print_table(&cat_headers, &cat_rows);
        }
    }

    Ok(())
}

fn query_repos(conn: &Connection) -> Result<Vec<RepoStat>, WlError> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(source_repo, 'unknown'), COUNT(*)
         FROM wordlists
         GROUP BY source_repo
         ORDER BY COUNT(*) DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(RepoStat {
            repository: r.get(0)?,
            count: r.get(1)?,
        })
    })?;
    let mut res = Vec::new();
    for r in rows {
        res.push(r?);
    }
    Ok(res)
}

fn query_categories(conn: &Connection) -> Result<Vec<CategoryStat>, WlError> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(category, '.'), COUNT(*)
         FROM wordlists
         GROUP BY category
         ORDER BY COUNT(*) DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(CategoryStat {
            category: r.get(0)?,
            count: r.get(1)?,
        })
    })?;
    let mut res = Vec::new();
    for r in rows {
        res.push(r?);
    }
    Ok(res)
}

fn query_largest(conn: &Connection) -> Result<Vec<LargestStat>, WlError> {
    let mut stmt = conn.prepare(
        "SELECT filename, size_bytes, COALESCE(source_repo, 'unknown'), path
         FROM wordlists
         ORDER BY size_bytes DESC
         LIMIT 20",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(LargestStat {
            filename: r.get(0)?,
            size_bytes: r.get(1)?,
            repo: r.get(2)?,
            path: r.get(3)?,
        })
    })?;
    let mut res = Vec::new();
    for r in rows {
        res.push(r?);
    }
    Ok(res)
}

fn query_extensions(conn: &Connection) -> Result<Vec<ExtensionStat>, WlError> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(extension, 'none'), COUNT(*)
         FROM wordlists
         GROUP BY extension
         ORDER BY COUNT(*) DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(ExtensionStat {
            extension: r.get(0)?,
            count: r.get(1)?,
        })
    })?;
    let mut res = Vec::new();
    for r in rows {
        res.push(r?);
    }
    Ok(res)
}
