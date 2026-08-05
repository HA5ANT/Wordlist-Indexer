use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn get_bin_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove file name
    if path.ends_with("deps") {
        path.pop();
    }
    path.join("wl")
}

#[test]
fn test_integration_flow() {
    // Generate a unique temp dir prefix
    let test_dir = std::env::temp_dir().join(format!("wl_test_{}", uuid_like_id()));
    fs::create_dir_all(&test_dir).unwrap();

    let config_path = test_dir.join("config.toml");
    let db_path = test_dir.join("index.db");
    let repo_dir = test_dir.join("wordlists");
    fs::create_dir_all(&repo_dir).unwrap();

    // Create fake wordlist files
    let rockyou_path = repo_dir.join("rockyou.txt");
    fs::write(&rockyou_path, "password\n123456\nqwerty\n").unwrap();

    let common_path = repo_dir.join("common.txt");
    fs::write(&common_path, "admin\nroot\nuser\n").unwrap();

    let bin = get_bin_path();

    let run_cmd = |args: &[&str]| {
        Command::new(&bin)
            .args(args)
            .env("WL_CONFIG_PATH", &config_path)
            .env("WL_DB_PATH", db_path.to_str().unwrap())
            .output()
            .unwrap()
    };

    // 1. Add repository
    let output = run_cmd(&["config", "add-repo", repo_dir.to_str().unwrap()]);
    assert!(output.status.success());

    // 2. Index
    let output = run_cmd(&["index"]);
    assert!(output.status.success());

    // 3. Exact search (wl rockyou)
    let output = run_cmd(&["rockyou"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), rockyou_path.to_str().unwrap());

    // 4. Fuzzy search (wl search rock)
    let output = run_cmd(&["search", "rock"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("rockyou.txt"));

    // 5. Non-existent wordlist lookup
    let output = run_cmd(&["nonexistent"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Wordlist not found"));

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

fn uuid_like_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", start)
}
