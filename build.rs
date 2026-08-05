use clap::CommandFactory;
use clap_complete::{generate_to, Shell};
use std::fs;
use std::path::Path;

#[path = "src/cli.rs"]
mod cli;

fn main() {
    let out_dir = match std::env::var_os("OUT_DIR") {
        Some(out) => out,
        None => return,
    };
    let out_path = Path::new(&out_dir).join("completions");
    fs::create_dir_all(&out_path).unwrap();

    let mut cmd = cli::Cli::command();
    for shell in &[Shell::Bash, Shell::Zsh, Shell::Fish] {
        generate_to(*shell, &mut cmd, "wl", &out_path).unwrap();
    }

    println!("cargo:rerun-if-changed=src/cli.rs");
}
