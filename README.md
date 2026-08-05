# wl (Wordlist Indexer)

A production-ready local search engine and metadata indexer for wordlists. Designed for penetration testers and CTF players to query wordlists instantly and embed them directly into security workflows.

## Features

- **Blazing Fast**: Initial execution in under 5 milliseconds.
- **Incremental Indexing**: Uses file sizes and `mtime` modification checks to skip unchanged files.
- **Fuzzy Search**: Fuzzy filename searches powered by `fuzzy-matcher`'s Skim algorithm.
- **Shell Friendly**: Outputs plain paths for easy pipeline chaining, or JSON/Table for reports.

---

## Installation & Setup

### Requirements
- Rust (Cargo) 1.70+

### Building

To build the debug version of the binary:
```bash
cargo build
```

To compile the optimized release build:
```bash
cargo build --release
```

### Installing the Binary
To install the compiled binary globally (to `/usr/local/bin`):
```bash
sudo cp target/release/wl /usr/local/bin/
```
Ensure `/usr/local/bin` is in your shell's `$PATH`.

---

## File Locations

- **Configuration File**: `~/.config/wl/config.toml`
- **Database File**: `~/.local/share/wl/index.db`

---

## Shell Completions Installation

During compiling, `wl` generates shell completion files under the build target's directory (specifically inside the `target/.../completions/` directory).

To install these manually:

### Bash
```bash
cp target/release/build/wl-*/out/completions/wl.bash ~/.local/share/bash-completion/completions/wl
```

### Zsh
```bash
cp target/release/build/wl-*/out/completions/_wl ~/.zsh/completion/_wl
```
*Note: Make sure your `fpath` variable contains `~/.zsh/completion`.*

### Fish
```bash
cp target/release/build/wl-*/out/completions/wl.fish ~/.config/fish/completions/wl.fish
```

---

## Example Usage

### 1. Add Wordlist Repositories
Add directories containing your wordlists:
```bash
wl config add-repo /usr/share/wordlists
wl config add-repo ~/hacking/wordlists
```

### 2. View Configuration
Verify configured repositories and the database path:
```bash
wl config list
```

### 3. Run the Indexer
Recursively scan and index wordlists:
```bash
wl index
```

### 4. Lookup Wordlists (Exact Match)
Query for a wordlist by its filename or stem:
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
Perform a fuzzy search on filename entries:
```bash
wl search raft-medium
```

### 6. List Wordlists
List all indexed wordlists with optional repository or extension filters:
```bash
wl ls --repo wordlists --ext txt
```

### 7. View Detailed Info
Print full metadata for an indexed wordlist:
```bash
wl info rockyou
```

### 8. Global Flags
All commands support global flags to control formatting:
- `--json`: Output as raw JSON arrays
- `--table`: Output as a neatly aligned terminal table
- `--quiet` / `-q`: Mute progress/logs on stderr
