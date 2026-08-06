# wl (Wordlist Indexer)

A production-ready local search engine and metadata indexer for wordlists. Designed for penetration testers and CTF players to query wordlists instantly and embed them directly into security workflows.

## Features

- **Blazing Fast**: Initial execution in under 5 milliseconds on lookup paths.
- **Incremental Indexing**: Skips unchanged files using `mtime` + `size_bytes` caching. SHA-256 hashes stored in the database for instant duplicate detection.
- **Fuzzy Search**: Fuzzy filename searches powered by `fuzzy-matcher`'s Skim algorithm.
- **Shell Friendly**: Outputs plain paths for pipeline chaining, or JSON/Table for reports.
- **Full Index Maintenance**: Stats, duplicate detection, integrity verification, and stale entry cleanup.

---

## Command Reference

| Command                      | Purpose                              |
| ---------------------------- | ------------------------------------ |
| `wl <name>`                  | Exact lookup — print path(s)         |
| `wl search <query>`          | Fuzzy search                         |
| `wl index`                   | Full rebuild of the index            |
| `wl update`                  | Incremental update (preserves manual tags) |
| `wl stats`                   | Collection statistics                |
| `wl duplicates`              | Find duplicate wordlists             |
| `wl tag add <id> <tag>`      | Manually add a tag to an entry       |
| `wl tag rm <id> <tag>`       | Remove a tag from an entry           |
| `wl verify`                  | Verify index integrity               |
| `wl remove-missing`          | Remove stale entries from index      |
| `wl info <name>`             | Show full metadata (includes tags)   |
| `wl ls`                      | Browse indexed wordlists             |
| `wl config add-repo <path>`  | Add a repository to scan             |
| `wl config remove-repo <p>`  | Remove a repository                  |
| `wl config list`             | Show config and db path              |

---

## Installation & Lifecycle Management

The `install.sh` script automates installation, upgrade, and removal of `wl` and its configurations. Quality checks (`cargo fmt --check`, `cargo clippy`, `cargo test`) are run automatically before any build.

### Requirements
- Rust (Cargo) 1.70+

### Install
Build the release binary, install to `/usr/local/bin`, create config directories, and install shell completions:
```bash
./install.sh install
# Or simply:
./install.sh
```

### Update
Update `wl` in-place while preserving your database, configuration, and indexed repositories:
```bash
./install.sh update
```
Example output:
```
Current version: 0.1.0
Installing:      0.2.0

Update complete.
```
Schema upgrades are handled by the application itself: on first run after an update, `wl` detects the
database version, applies additive migrations, and backs up the database to `index.db.bak` before any
change — no manual migration steps required.

### Uninstall
Completely remove `wl`, its completions, and (optionally) its config and database:
```bash
./install.sh uninstall
```
Example output:
```
Configuration and index database found.

Delete them? [y/N]
```

---

## File Locations

- **Configuration File**: `~/.config/wl/config.toml`
- **Database File**: `~/.local/share/wl/index.db`

---

## Shell Completions

Completions are generated at compile time into `target/release/build/wl-*/out/completions/`.
The installer places them automatically. To install manually:

### Bash
```bash
cp target/release/build/wl-*/out/completions/wl.bash \
   /usr/share/bash-completion/completions/wl
```

### Zsh
```bash
cp target/release/build/wl-*/out/completions/_wl \
   /usr/local/share/zsh/site-functions/_wl
```

### Fish
```bash
cp target/release/build/wl-*/out/completions/wl.fish \
   ~/.config/fish/completions/wl.fish
```

---

## Example Usage

### 1. Add Wordlist Repositories
```bash
wl config add-repo /usr/share/wordlists
wl config add-repo ~/hacking/wordlists
wl config list
```

### 2. Full Index
Recursively scan and index all repos:
```bash
wl index
```

### 3. Incremental Update
Scan repos incrementally — skip unchanged files, insert new ones, remove deleted ones. Manual tags
added via `wl tag add` are preserved, even for files that were modified:
```bash
wl update
```

### 4. Exact Lookup
Query by filename or stem (case-insensitive):
```bash
wl rockyou
# Output: /usr/share/wordlists/rockyou.txt
```

#### Shell Integration Examples
```bash
ffuf -w "$(wl raft-medium-directories)"
hashcat -m 0 hashes.txt "$(wl rockyou)"
gobuster dir -w "$(wl common.txt)"
```

### 5. Fuzzy Search
```bash
wl search raft-medium
```

### 6. Statistics
```bash
wl stats                  # Full overview
wl stats --repo           # By repository
wl stats --category       # By category
wl stats --largest        # Top 20 largest files
wl stats --extensions     # By extension
wl stats --json           # JSON output
```

### 7. Duplicate Detection
```bash
wl duplicates             # Group by filename (default)
wl duplicates --name      # Group by filename
wl duplicates --size      # Group by size (fast heuristic)
wl duplicates --hash      # Group by SHA-256 (exact match)
```

### 8. Verify Index Integrity
```bash
wl verify
```
Example output:
```
✓ Database readable
✓ SQLite integrity OK

6483 entries checked

✓ 6480 files exist

⚠ 3 missing files

✓ 0 unreadable files

✓ 0 duplicate DB entries

✓ 0 broken symlinks

PASS
```

### 9. Remove Stale Entries
Remove index entries for files that no longer exist on disk (without rescanning):
```bash
wl remove-missing
# Removed 3 stale entries.
```

### 10. List & Filter
```bash
wl ls                         # All indexed wordlists
wl ls --repo wordlists        # Filter by repository name
wl ls --ext txt               # Filter by extension
wl ls --tag web,fuzzing       # Filter by tags (OR logic: entries with ANY of the specified tags)
wl ls --table                 # Table format
wl ls --json                  # JSON output
```

### 11. View Full Metadata & Tags
```bash
wl info rockyou
# Output now includes a "Tags: ..." line
```

### 12. Tag Management
```bash
# Find entry ID from `wl info` or `wl ls --table`
wl tag add 123 custom-tag
wl tag rm 123 custom-tag
```

### 13. Advanced Fuzzy Search with Tags
```bash
wl search raft --tag webcontent
```

### 14. Global Flags
All commands support:
- `--json`: Output as raw JSON
- `--table`: Output as aligned table
- `--quiet` / `-q`: Suppress all stderr output
