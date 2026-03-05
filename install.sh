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
    fi
}

main
