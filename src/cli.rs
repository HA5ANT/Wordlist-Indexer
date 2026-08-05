use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "wl",
    version = "0.1.0",
    about = "Wordlist Indexer",
    arg_required_else_help = true
)]
pub struct Cli {
    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Output as formatted table
    #[arg(long, global = true)]
    pub table: bool,

    /// Suppress all stderr output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Wordlist name for exact lookup (default command)
    #[arg(value_name = "NAME")]
    pub name: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Fuzzy search wordlists
    Search {
        /// Query to search for
        query: String,
    },
    /// Scan and index wordlists
    Index {
        /// Optional specific path to scan and index
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// List indexed wordlists with optional filters
    Ls {
        /// Filter by repository name
        #[arg(long)]
        repo: Option<String>,

        /// Filter by file extension
        #[arg(long)]
        ext: Option<String>,
    },
    /// Display full metadata for a wordlist
    Info {
        /// Name of the wordlist
        name: String,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        subaction: ConfigSubcommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcommands {
    /// Add a directory to index
    AddRepo {
        /// Path of the directory
        path: PathBuf,
    },
    /// Remove a directory from index
    RemoveRepo {
        /// Path of the directory
        path: PathBuf,
    },
    /// Show current repos and db path
    List,
}
