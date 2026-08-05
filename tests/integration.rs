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

    let repo2_dir = test_dir.join("wordlists2");
    fs::create_dir_all(&repo2_dir).unwrap();

    // Create fake wordlist files
    let rockyou_path = repo_dir.join("rockyou.txt");
    fs::write(&rockyou_path, "password\n123456\nqwerty\n").unwrap();

    let common_path = repo_dir.join("common.txt");
    fs::write(&common_path, "admin\nroot\nuser\n").unwrap();

    // Create duplicate file (matching filename and content hash)
    let dup_rockyou_path = repo2_dir.join("rockyou.txt");
    fs::write(&dup_rockyou_path, "password\n123456\nqwerty\n").unwrap();

    let bin = get_bin_path();

    let run_cmd = |args: &[&str]| {
        Command::new(&bin)
            .args(args)
            .env("WL_CONFIG_PATH", &config_path)
            .env("WL_DB_PATH", db_path.to_str().unwrap())
            .output()
            .unwrap()
    };

    // 1. Add repositories
    let output = run_cmd(&["config", "add-repo", repo_dir.to_str().unwrap()]);
    assert!(output.status.success());
    let output = run_cmd(&["config", "add-repo", repo2_dir.to_str().unwrap()]);
    assert!(output.status.success());

    // 2. Full index
    let output = run_cmd(&["index"]);
    assert!(output.status.success());

    // 3. Exact search (wl rockyou) - NEW BEHAVIOR: 1 path on stdout, note on stderr
    let output = run_cmd(&["rockyou"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stdout.lines().count(), 1);
    assert!(stderr.contains("note: also matches"));

    // 4. Stats
    let output = run_cmd(&["stats"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Number of repositories"));
    assert!(stdout.contains("Number of indexed wordlists"));

    let output = run_cmd(&["stats", "--repo"]);
    assert!(output.status.success());
    let output = run_cmd(&["stats", "--category"]);
    assert!(output.status.success());
    let output = run_cmd(&["stats", "--largest"]);
    assert!(output.status.success());
    let output = run_cmd(&["stats", "--extensions"]);
    assert!(output.status.success());

    // 5. Duplicate detection
    let output = run_cmd(&["duplicates", "--name"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("rockyou.txt"));

    let output = run_cmd(&["duplicates", "--size"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("rockyou.txt"));

    let output = run_cmd(&["duplicates", "--hash"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("rockyou.txt"));

    // 6. Verify index
    let output = run_cmd(&["verify"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Database readable"));
    assert!(stdout.contains("SQLite integrity OK"));
    assert!(stdout.contains("PASS"));

    // 7. Update after file modification
    // Get info first
    let output = run_cmd(&["info", "common"]);
    assert!(output.status.success());
    let info_stdout = String::from_utf8(output.stdout).unwrap();

    // Modify file
    fs::write(&common_path, "admin\nroot\nuser\nmodified_content\n").unwrap();

    // Run update (incremental)
    let output = run_cmd(&["update"]);
    assert!(output.status.success());

    let output = run_cmd(&["info", "common"]);
    assert!(output.status.success());
    let new_info_stdout = String::from_utf8(output.stdout).unwrap();
    assert_ne!(info_stdout, new_info_stdout);

    // 8. Update after file deletion
    fs::remove_file(&dup_rockyou_path).unwrap();
    let output = run_cmd(&["update"]);
    assert!(output.status.success());

    // rockyou should now have only 1 exact match path
    let output = run_cmd(&["rockyou"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), rockyou_path.to_str().unwrap());

    // 9. remove-missing command
    // Manually delete common.txt from file system without running update first
    fs::remove_file(&common_path).unwrap();

    let output = run_cmd(&["remove-missing"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Removed 1 stale entries."));

    // Verify it is gone
    let output = run_cmd(&["common"]);
    assert!(!output.status.success());

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
