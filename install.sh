#!/bin/sh
set -eu

REPO="sounak98/betterstack-cli"
INSTALL_DIR="${BS_INSTALL_DIR:-$HOME/.local/bin}"

main() {
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$os" in
        linux)  os="unknown-linux-gnu" ;;
        darwin) os="apple-darwin" ;;
        *) echo "Unsupported OS: $os" >&2; exit 1 ;;
    esac

    case "$arch" in
        x86_64|amd64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
    esac

    target="${arch}-${os}"

    if [ -n "${BS_VERSION:-}" ]; then
        tag="v$BS_VERSION"
    else
        tag=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"//;s/".*//')
        if [ -z "$tag" ]; then
            echo "Failed to fetch latest release" >&2
            exit 1
        fi
    fi

    url="https://github.com/$REPO/releases/download/$tag/bs-$target.tar.gz"

    echo "Installing bs $tag ($target)"
    echo "  from: $url"
    echo "  to:   $INSTALL_DIR/bs"
    echo

    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT

    curl -fsSL "$url" -o "$tmp/bs.tar.gz"
    tar xzf "$tmp/bs.tar.gz" -C "$tmp"

    mkdir -p "$INSTALL_DIR"
    mv "$tmp/bs" "$INSTALL_DIR/bs"
    chmod +x "$INSTALL_DIR/bs"

    echo "Installed bs $tag successfully!"
    echo

    if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
        echo "Add $INSTALL_DIR to your PATH:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        echo
    fi

    install_completions
}

install_completions() {
    shell_name=$(basename "${SHELL:-}")

    case "$shell_name" in
        bash)
            comp_dir="${BASH_COMPLETION_USER_DIR:-$HOME/.local/share/bash-completion/completions}"
            mkdir -p "$comp_dir"
            "$INSTALL_DIR/bs" completions bash > "$comp_dir/bs"
            echo "Installed bash completions to $comp_dir/bs"
            ;;
        zsh)
            comp_dir="${HOME}/.zfunc"
            mkdir -p "$comp_dir"
            "$INSTALL_DIR/bs" completions zsh > "$comp_dir/_bs"
            echo "Installed zsh completions to $comp_dir/_bs"
            if ! grep -q 'fpath.*\.zfunc' "${ZDOTDIR:-$HOME}/.zshrc" 2>/dev/null; then
                echo "Add this to your .zshrc (before compinit):"
                echo "  fpath=(~/.zfunc \$fpath)"
            fi
            ;;
        fish)
            comp_dir="${HOME}/.config/fish/completions"
            mkdir -p "$comp_dir"
            "$INSTALL_DIR/bs" completions fish > "$comp_dir/bs.fish"
            echo "Installed fish completions to $comp_dir/bs.fish"
            ;;
        *)
            echo "Shell completions available via: bs completions <bash|zsh|fish|powershell|elvish>"
            ;;
    esac
}

main
