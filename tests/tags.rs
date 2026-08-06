use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn get_bin_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.join("wl")
}

#[test]
fn test_phase3_tags() {
    let test_dir = std::env::temp_dir().join(format!("wl_tags_{}", uuid_like_id()));
    fs::create_dir_all(&test_dir).unwrap();

    let config_path = test_dir.join("config.toml");
    let db_path = test_dir.join("index.db");

    let repo = test_dir.join("repo");
    fs::create_dir_all(repo.join("Discovery/Web-Content")).unwrap();
    fs::create_dir_all(repo.join("Fuzzing/XSS")).unwrap();
    fs::create_dir_all(repo.join("Passwords")).unwrap();

    fs::write(repo.join("Discovery/Web-Content/common.txt"), "admin\n").unwrap();
    fs::write(repo.join("Fuzzing/XSS/payload.txt"), "<script>\n").unwrap();
    fs::write(repo.join("Passwords/rockyou.txt"), "password\n").unwrap();

    let bin = get_bin_path();
    let run_cmd = |args: &[&str]| {
        Command::new(&bin)
            .args(args)
            .env("WL_CONFIG_PATH", &config_path)
            .env("WL_DB_PATH", db_path.to_str().unwrap())
            .output()
            .unwrap()
    };

    run_cmd(&["config", "add-repo", repo.to_str().unwrap()]);
    let out = run_cmd(&["index"]);
    assert!(
        out.status.success(),
        "index failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // 1. Auto-tagging: hyphenated dirs must match the taxonomy
    let out = run_cmd(&["info", "common"]);
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("webcontent"),
        "missing webcontent tag: {stdout}"
    );
    assert!(
        stdout.contains("discovery"),
        "missing discovery tag: {stdout}"
    );
    assert!(stdout.contains("web"), "missing web tag: {stdout}");

    let out = run_cmd(&["info", "payload"]);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("xss"), "missing xss tag: {stdout}");
    assert!(stdout.contains("fuzzing"), "missing fuzzing tag: {stdout}");
    assert!(
        stdout.contains("injection"),
        "missing injection tag: {stdout}"
    );

    // 2. Manual tag survives an unchanged update
    let out = run_cmd(&["ls", "--table"]);
    let table = String::from_utf8(out.stdout).unwrap();
    let common_id = table
        .lines()
        .find(|l| l.contains("common.txt"))
        .and_then(|l| l.split_whitespace().next())
        .expect("common.txt row in table");
    let out = run_cmd(&["tag", "add", common_id, "my-custom"]);
    assert!(out.status.success());

    let out = run_cmd(&["update"]);
    assert!(out.status.success());

    let out = run_cmd(&["info", "common"]);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("my-custom"),
        "manual tag lost after update: {stdout}"
    );

    // 3. Manual tag survives a modified file + update, and id stays stable
    let before_id = entry_id(&db_path, "common.txt");
    fs::write(
        repo.join("Discovery/Web-Content/common.txt"),
        "admin\nroot\n",
    )
    .unwrap();
    let out = run_cmd(&["update"]);
    assert!(out.status.success());
    let after_id = entry_id(&db_path, "common.txt");
    assert_eq!(before_id, after_id, "entry id changed across update");
    let out = run_cmd(&["info", "common"]);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("my-custom"),
        "manual tag lost after modified update: {stdout}"
    );

    // 4. Manual tag survives a full index (rebuild)
    let out = run_cmd(&["index"]);
    assert!(out.status.success());
    let out = run_cmd(&["info", "common"]);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("my-custom"),
        "manual tag lost after full index: {stdout}"
    );

    // 5. Tag filter is OR across multiple tags
    let out = run_cmd(&["ls", "--tag", "xss,passwords"]);
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("payload.txt"),
        "xss entry missing from OR filter: {stdout}"
    );
    assert!(
        stdout.contains("rockyou.txt"),
        "passwords entry missing from OR filter: {stdout}"
    );
    assert!(
        !stdout.contains("common.txt"),
        "untagged entry should be excluded by OR filter: {stdout}"
    );

    fs::remove_dir_all(&test_dir).unwrap();
}

fn entry_id(db_path: &PathBuf, filename: &str) -> Option<String> {
    let out = Command::new("sqlite3")
        .arg(db_path.to_str().unwrap())
        .arg(format!(
            "SELECT id FROM wordlists WHERE filename = '{filename}'"
        ))
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

fn uuid_like_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", start)
}
