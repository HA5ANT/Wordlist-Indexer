mod cli;
mod config;
mod db;
mod duplicates;
mod error;
mod indexer;
mod output;
mod search;
mod stats;
mod updater;
mod verify;

use clap::Parser;
use cli::{Cli, Commands, ConfigSubcommands};
use error::WlError;
use output::OutputMode;
use std::fs;
use std::process;

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), WlError> {
    let cli = Cli::parse();

    let mode = if cli.json {
        OutputMode::Json
    } else if cli.table {
        OutputMode::Table
    } else {
        OutputMode::Plain
    };

    let config = config::load()?;
    let db_path = config::expand_tilde(&config.db_path);

    if let Some(Commands::Config { subaction }) = &cli.command {
        match subaction {
            ConfigSubcommands::AddRepo { path } => {
                config::add_repo(path.clone())?;
                if !cli.quiet {
                    eprintln!("[+] Added repository: {}", path.display());
                }
            }
            ConfigSubcommands::RemoveRepo { path } => {
                config::remove_repo(path.clone())?;
                if !cli.quiet {
                    eprintln!("[-] Removed repository: {}", path.display());
                }
            }
            ConfigSubcommands::List => {
                println!("Database Path: {}", db_path.display());
                println!("Repositories:");
                if config.repos.is_empty() {
                    println!("  (none)");
                } else {
                    for repo in &config.repos {
                        println!("  - {}", repo.display());
                    }
                }
            }
        }
        return Ok(());
    }

    let conn = db::init(&db_path)?;

    if let Some(ref name) = cli.name {
        let results = search::exact_lookup(&conn, name)?;
        if results.is_empty() {
            return Err(WlError::NotFound(name.clone()));
        }

        if mode == OutputMode::Plain {
            if results.len() == 1 {
                println!("{}", results[0].path);
                process::exit(0);
            } else {
                for entry in &results {
                    println!("{}", entry.path);
                }
                process::exit(0);
            }
        } else {
            output::render_entries(&results, mode);
        }
        return Ok(());
    }

    match cli.command {
        Some(Commands::Search { query }) => {
            let results = search::fuzzy_search(&conn, &query)?;
            output::render_fuzzy(&results, mode);
        }
        Some(Commands::Index { path }) => {
            let repos_to_scan = if let Some(p) = path {
                vec![p]
            } else {
                if config.repos.is_empty() {
                    return Err(WlError::NoReposConfigured);
                }
                config.repos.clone()
            };
            indexer::index_full(&conn, &repos_to_scan, cli.quiet)?;
        }
        Some(Commands::Ls { repo, ext }) => {
            let all = db::get_all(&conn)?;
            let filtered: Vec<db::WordlistEntry> = all
                .into_iter()
                .filter(|e| {
                    if let Some(ref r) = repo {
                        if let Some(ref entry_repo) = e.source_repo {
                            if !entry_repo.eq_ignore_ascii_case(r) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    if let Some(ref x) = ext {
                        if let Some(ref entry_ext) = e.extension {
                            if !entry_ext.eq_ignore_ascii_case(x) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    true
                })
                .collect();
            output::render_entries(&filtered, mode);
        }
        Some(Commands::Info { name }) => {
            let results = search::exact_lookup(&conn, &name)?;
            if results.is_empty() {
                return Err(WlError::NotFound(name));
            }
            if matches!(mode, OutputMode::Json | OutputMode::Table) {
                output::render_entries(&results, mode);
            } else {
                for (idx, entry) in results.iter().enumerate() {
                    if idx > 0 {
                        println!();
                    }
                    println!("Filename:      {}", entry.filename);
                    println!("Stem:          {}", entry.stem);
                    println!("Path:          {}", entry.path);
                    println!(
                        "Extension:     {}",
                        entry.extension.as_deref().unwrap_or("-")
                    );

                    let size_str = if entry.size_bytes >= 1024 * 1024 {
                        format!(
                            "{:.1} MB ({} bytes)",
                            entry.size_bytes as f64 / (1024.0 * 1024.0),
                            entry.size_bytes
                        )
                    } else if entry.size_bytes >= 1024 {
                        format!(
                            "{:.1} KB ({} bytes)",
                            entry.size_bytes as f64 / 1024.0,
                            entry.size_bytes
                        )
                    } else {
                        format!("{} bytes", entry.size_bytes)
                    };
                    println!("Size:          {}", size_str);
                    println!(
                        "Repository:    {}",
                        entry.source_repo.as_deref().unwrap_or("-")
                    );
                    println!(
                        "Category:      {}",
                        entry.category.as_deref().unwrap_or("-")
                    );
                    println!(
                        "Compressed:    {}",
                        if entry.compressed { "Yes" } else { "No" }
                    );

                    let line_count_str = entry
                        .line_count
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    println!("Line Count:    {}", line_count_str);

                    let dt = chrono::DateTime::from_timestamp(entry.mtime, 0)
                        .map(|t| t.to_rfc3339())
                        .unwrap_or_else(|| "unknown".to_string());
                    println!("Last Modified: {}", dt);

                    let sha256_str = entry.sha256.as_deref().unwrap_or("-");
                    println!("SHA-256:       {}", sha256_str);
                }
            }
        }
        Some(Commands::Stats {
            repo,
            category,
            largest,
            extensions,
        }) => {
            stats::run_stats(&conn, repo, category, largest, extensions, mode)?;
        }
        Some(Commands::Duplicates {
            name: _,
            size,
            hash,
        }) => {
            let dup_mode = if size {
                duplicates::DupMode::Size
            } else if hash {
                duplicates::DupMode::Hash
            } else {
                duplicates::DupMode::Name
            };
            duplicates::run_duplicates(&conn, dup_mode, mode)?;
        }
        Some(Commands::Verify) => {
            verify::run_verify(&conn)?;
        }
        Some(Commands::Update { path }) => {
            let repos_to_scan = if let Some(p) = path {
                vec![p]
            } else {
                if config.repos.is_empty() {
                    return Err(WlError::NoReposConfigured);
                }
                config.repos.clone()
            };
            updater::update_incremental(&conn, &repos_to_scan, cli.quiet)?;
        }
        Some(Commands::RemoveMissing) => {
            let all = db::get_all(&conn)?;
            let mut removed = 0;
            for entry in all {
                if fs::metadata(&entry.path).is_err() {
                    db::delete_entry(&conn, &entry.path)?;
                    removed += 1;
                }
            }
            if !cli.quiet {
                println!("Removed {} stale entries.", removed);
            }
        }
        _ => {}
    }

    Ok(())
}
