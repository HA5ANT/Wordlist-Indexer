#!/usr/bin/env bash

# Robust shell settings
set -euo pipefail

echo "[+] Building release binary..."
cargo build --release

echo "[+] Installing binary to /usr/local/bin..."
sudo cp "target/release/wl" "/usr/local/bin/"

echo "[+] Creating configuration and data directories..."
mkdir -p "$HOME/.config/wl"
mkdir -p "$HOME/.local/share/wl"

echo "[+] Detecting generated shell completions..."
COMPLETIONS_DIR=$(find "target/release/build" -type d -name "completions" | head -n 1 || true)

if [ -n "${COMPLETIONS_DIR}" ] && [ -d "${COMPLETIONS_DIR}" ]; then
    echo "[+] Installing shell completions from ${COMPLETIONS_DIR}..."

    # Install Bash completions
    if [ -d "/usr/share/bash-completion/completions" ]; then
        BASH_FILE=$(find "${COMPLETIONS_DIR}" -type f \( -name "wl.bash" -o -name "wl" \) | head -n 1 || true)
        if [ -n "${BASH_FILE}" ]; then
            sudo cp "${BASH_FILE}" "/usr/share/bash-completion/completions/wl"
            echo "    * Installed Bash completions to /usr/share/bash-completion/completions/wl"
        else
            echo "    [!] Warning: Bash completion file not found in completions dir."
        fi
    fi

    # Install Zsh completions
    ZSH_DEST=""
    if [ -d "/usr/share/zsh/vendor-completions" ]; then
        ZSH_DEST="/usr/share/zsh/vendor-completions"
    elif [ -d "/usr/local/share/zsh/site-functions" ]; then
        ZSH_DEST="/usr/local/share/zsh/site-functions"
    fi

    if [ -n "${ZSH_DEST}" ]; then
        ZSH_FILE=$(find "${COMPLETIONS_DIR}" -type f -name "_wl" | head -n 1 || true)
        if [ -n "${ZSH_FILE}" ]; then
            sudo cp "${ZSH_FILE}" "${ZSH_DEST}/_wl"
            echo "    * Installed Zsh completions to ${ZSH_DEST}/_wl"
        else
            echo "    [!] Warning: Zsh completion file (_wl) not found in completions dir."
        fi
    fi

    # Install Fish completions
    FISH_DEST=""
    if [ -d "/usr/share/fish/vendor_completions.d" ]; then
        FISH_DEST="/usr/share/fish/vendor_completions.d"
    elif [ -d "/usr/local/share/fish/vendor_completions.d" ]; then
        FISH_DEST="/usr/local/share/fish/vendor_completions.d"
    fi

    if [ -n "${FISH_DEST}" ]; then
        FISH_FILE=$(find "${COMPLETIONS_DIR}" -type f -name "wl.fish" | head -n 1 || true)
        if [ -n "${FISH_FILE}" ]; then
            sudo cp "${FISH_FILE}" "${FISH_DEST}/wl.fish"
            echo "    * Installed Fish completions to ${FISH_DEST}/wl.fish"
        else
            echo "    [!] Warning: Fish completion file (wl.fish) not found in completions dir."
        fi
    fi
else
    echo "[!] Warning: Completion files directory not found. Skipping completion installation."
fi

echo "[+] Verifying installation..."
if ! command -v wl >/dev/null 2>&1; then
    echo "[!] Error: 'wl' command not found in PATH."
    exit 1
fi

WL_VER=$(wl --version || true)
echo "[+] Successfully installed wl. Version: ${WL_VER}"
