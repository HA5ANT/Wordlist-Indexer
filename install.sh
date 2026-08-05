#!/usr/bin/env bash

# Robust shell settings
set -euo pipefail

ACTION="${1:-install}"

get_completions_dir() {
    find "target/release/build" -type d -name "completions" | head -n 1 || true
}

verify_quality() {
    echo "[+] Running quality verification checks..."
    cargo fmt --check
    cargo clippy -- -D warnings
    cargo test
}

install_action() {
    verify_quality

    echo "[+] Building release binary..."
    cargo build --release

    echo "[+] Installing binary to /usr/local/bin..."
    sudo cp "target/release/wl" "/usr/local/bin/"

    echo "[+] Creating configuration and data directories..."
    mkdir -p "$HOME/.config/wl"
    mkdir -p "$HOME/.local/share/wl"

    echo "[+] Detecting generated shell completions..."
    COMPLETIONS_DIR=$(get_completions_dir)

    if [ -n "${COMPLETIONS_DIR}" ] && [ -d "${COMPLETIONS_DIR}" ]; then
        # Bash
        if [ -d "/usr/share/bash-completion/completions" ]; then
            BASH_FILE=$(find "${COMPLETIONS_DIR}" -type f \( -name "wl.bash" -o -name "wl" \) | head -n 1 || true)
            if [ -n "${BASH_FILE}" ]; then
                echo "[+] Installing Bash completions..."
                sudo cp "${BASH_FILE}" "/usr/share/bash-completion/completions/wl"
            else
                echo "[!] Bash completion file not found. Skipping."
            fi
        else
            echo "[!] Bash completion directory not found. Skipping."
        fi

        # Zsh
        ZSH_DEST=""
        if [ -d "/usr/share/zsh/vendor-completions" ]; then
            ZSH_DEST="/usr/share/zsh/vendor-completions"
        elif [ -d "/usr/local/share/zsh/site-functions" ]; then
            ZSH_DEST="/usr/local/share/zsh/site-functions"
        fi

        if [ -n "${ZSH_DEST}" ]; then
            ZSH_FILE=$(find "${COMPLETIONS_DIR}" -type f -name "_wl" | head -n 1 || true)
            if [ -n "${ZSH_FILE}" ]; then
                echo "[+] Installing Zsh completions..."
                sudo cp "${ZSH_FILE}" "${ZSH_DEST}/_wl"
            else
                echo "[!] Zsh completion file (_wl) not found. Skipping."
            fi
        else
            echo "[!] Zsh completion directory not found. Skipping."
        fi

        # Fish
        FISH_DEST=""
        if [ -d "/usr/share/fish/vendor_completions.d" ]; then
            FISH_DEST="/usr/share/fish/vendor_completions.d"
        elif [ -d "/usr/local/share/fish/vendor_completions.d" ]; then
            FISH_DEST="/usr/local/share/fish/vendor_completions.d"
        fi

        if [ -n "${FISH_DEST}" ]; then
            FISH_FILE=$(find "${COMPLETIONS_DIR}" -type f -name "wl.fish" | head -n 1 || true)
            if [ -n "${FISH_FILE}" ]; then
                echo "[+] Installing Fish completions..."
                sudo cp "${FISH_FILE}" "${FISH_DEST}/wl.fish"
            else
                echo "[!] Fish completion file (wl.fish) not found. Skipping."
            fi
        else
            echo "[!] Fish completion directory not found. Skipping."
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
}

update_action() {
    PREV_VERSION="none"
    if command -v wl >/dev/null 2>&1; then
        PREV_VERSION=$(wl --version | awk '{print $2}' || wl --version || true)
    fi

    verify_quality

    echo "[+] Building release binary..."
    cargo build --release

    NEW_VERSION=$(target/release/wl --version | awk '{print $2}' || target/release/wl --version || true)

    echo "Current version: ${PREV_VERSION}"
    echo "Installing:      ${NEW_VERSION}"

    echo "[+] Replacing installed binary..."
    sudo cp "target/release/wl" "/usr/local/bin/"

    echo "[+] Refreshing shell completions..."
    COMPLETIONS_DIR=$(get_completions_dir)

    if [ -n "${COMPLETIONS_DIR}" ] && [ -d "${COMPLETIONS_DIR}" ]; then
        if [ -d "/usr/share/bash-completion/completions" ]; then
            BASH_FILE=$(find "${COMPLETIONS_DIR}" -type f \( -name "wl.bash" -o -name "wl" \) | head -n 1 || true)
            if [ -n "${BASH_FILE}" ]; then
                sudo cp "${BASH_FILE}" "/usr/share/bash-completion/completions/wl"
            fi
        fi

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
            fi
        fi

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
            fi
        fi
    fi

    echo ""
    echo "Update complete."
}

uninstall_action() {
    echo "[+] Removing binary..."
    if [ -f "/usr/local/bin/wl" ]; then
        sudo rm -f "/usr/local/bin/wl"
        echo "    * Removed /usr/local/bin/wl"
    fi

    echo "[+] Removing shell completions..."
    if [ -f "/usr/share/bash-completion/completions/wl" ]; then
        sudo rm -f "/usr/share/bash-completion/completions/wl"
    fi

    if [ -f "/usr/share/zsh/vendor-completions/_wl" ]; then
        sudo rm -f "/usr/share/zsh/vendor-completions/_wl"
    elif [ -f "/usr/local/share/zsh/site-functions/_wl" ]; then
        sudo rm -f "/usr/local/share/zsh/site-functions/_wl"
    fi

    if [ -f "/usr/share/fish/vendor_completions.d/wl.fish" ]; then
        sudo rm -f "/usr/share/fish/vendor_completions.d/wl.fish"
    elif [ -f "/usr/local/share/fish/vendor_completions.d/wl.fish" ]; then
        sudo rm -f "/usr/local/share/fish/vendor_completions.d/wl.fish"
    fi

    if [ -d "$HOME/.config/wl" ] || [ -d "$HOME/.local/share/wl" ]; then
        echo ""
        echo "Configuration and index database found."
        echo ""
        read -r -p "Delete them? [y/N] " response
        if [[ "$response" =~ ^[Yy]$ ]]; then
            rm -rf "$HOME/.config/wl"
            rm -rf "$HOME/.local/share/wl"
            echo "[+] Configuration and index database removed."
        else
            echo "[+] Preserved configuration and database."
        fi
    fi

    echo ""
    echo "Uninstall complete."
}

case "${ACTION}" in
    install)
        install_action
        ;;
    update)
        update_action
        ;;
    uninstall)
        uninstall_action
        ;;
    *)
        echo "Usage: $0 {install|update|uninstall}"
        exit 1
        ;;
esac
