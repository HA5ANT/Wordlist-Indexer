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
fn test_name_resolution() {
    // Setup
    let test_dir = std::env::temp_dir().join(format!("wl_res_{}", uuid_like_id()));
    fs::create_dir_all(&test_dir).unwrap();
    let config_path = test_dir.join("config.toml");
    let db_path = test_dir.join("index.db");

    let repo1 = test_dir.join("repo1");
    let repo2 = test_dir.join("repo2");
    fs::create_dir_all(&repo1).unwrap();
    fs::create_dir_all(&repo2).unwrap();

    // Create files
    fs::write(repo1.join("dup.txt"), "content1").unwrap();
    fs::write(repo2.join("dup.txt"), "content2").unwrap();
    fs::write(repo1.join("fuzzy_match_target.txt"), "content").unwrap();

    let bin = get_bin_path();
    let run_cmd = |args: &[&str]| {
        Command::new(&bin)
            .args(args)
            .env("WL_CONFIG_PATH", &config_path)
            .env("WL_DB_PATH", db_path.to_str().unwrap())
            .output()
            .unwrap()
    };

    run_cmd(&["config", "add-repo", repo1.to_str().unwrap()]);
    run_cmd(&["config", "add-repo", repo2.to_str().unwrap()]);
    run_cmd(&["index"]);

    // 1. Multi exact (searching for "dup")
    let output = run_cmd(&["dup"]);
    assert!(
        output.status.success(),
        "exact lookup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stdout.lines().count(), 1);
    assert!(stderr.contains("note: also matches"));

    // 2. Fuzzy clear winner
    let output = run_cmd(&["fuzzy_match_targ"]);
    assert!(
        output.status.success(),
        "fuzzy lookup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("fuzzy_match_target.txt"));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("note: fuzzy match"));

    // 3. No match
    let output = run_cmd(&["nonexistent"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Wordlist not found: nonexistent"));

    fs::remove_dir_all(&test_dir).unwrap();
}

fn uuid_like_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", start)
}
