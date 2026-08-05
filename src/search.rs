use crate::db::{self, WordlistEntry};
use crate::error::WlError;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rusqlite::Connection;

pub const FUZZY_MIN_SCORE: i64 = 50; // Need tuning
pub const FUZZY_MIN_GAP: i64 = 20; // Need tuning

#[derive(Debug, Clone)]
pub struct FuzzyResult {
    pub score: i64,
    pub filename: String,
    pub repo: String,
    pub path: String,
}

pub fn exact_lookup(conn: &Connection, name: &str) -> Result<Vec<WordlistEntry>, WlError> {
    db::get_by_name(conn, name)
}

pub fn fuzzy_search(conn: &Connection, query: &str) -> Result<Vec<FuzzyResult>, WlError> {
    let all_entries = db::get_all(conn)?;
    let matcher = SkimMatcherV2::default();

    let mut results: Vec<FuzzyResult> = all_entries
        .into_iter()
        .filter_map(|entry| {
            matcher
                .fuzzy_match(&entry.filename, query)
                .map(|score| FuzzyResult {
                    score,
                    filename: entry.filename,
                    repo: entry.source_repo.unwrap_or_else(|| "unknown".to_string()),
                    path: entry.path,
                })
        })
        .collect();

    results.sort_by(|a, b| {
        let cmp = b.score.cmp(&a.score);
        if cmp == std::cmp::Ordering::Equal {
            a.path.cmp(&b.path)
        } else {
            cmp
        }
    });

    results.truncate(10);
    Ok(results)
}
