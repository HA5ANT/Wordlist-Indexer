use thiserror::Error;

#[derive(Error, Debug)]
pub enum WlError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Wordlist not found: {0}")]
    NotFound(String),

    #[error("No repos configured. Run: wl config add-repo <path>")]
    NoReposConfigured,
}
