#!/bin/bash
# Install script for cc-gateway
# Usage: curl -fsSL https://raw.githubusercontent.com/user/cc-gateway/main/install.sh | bash

set -e

REPO="user/cc-gateway"
BINARY_NAME="cc-gateway"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)
            if [ "$ARCH" = "x86_64" ]; then
                PLATFORM="x86_64-unknown-linux-musl"
            elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
                PLATFORM="aarch64-unknown-linux-musl"
            else
                echo "Unsupported architecture: $ARCH"
                exit 1
            fi
            ;;
        Darwin*)
            if [ "$ARCH" = "x86_64" ]; then
                PLATFORM="x86_64-apple-darwin"
            elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
                PLATFORM="aarch64-apple-darwin"
            else
                echo "Unsupported architecture: $ARCH"
                exit 1
            fi
            ;;
        MINGW* | MSYS* | CYGWIN*)
            if [ "$ARCH" = "x86_64" ]; then
                PLATFORM="x86_64-pc-windows-msvc"
                BINARY_NAME="cc-gateway.exe"
            elif [ "$ARCH" = "aarch64" ]; then
                PLATFORM="aarch64-pc-windows-msvc"
                BINARY_NAME="cc-gateway.exe"
            else
                echo "Unsupported architecture: $ARCH"
                exit 1
            fi
            ;;
        *)
            echo "Unsupported OS: $OS"
            exit 1
            ;;
    esac
}

# Get latest version from GitHub API
get_latest_version() {
    if command -v curl &> /dev/null; then
        VERSION=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')
    elif command -v wget &> /dev/null; then
        VERSION=$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')
    else
        echo "Error: curl or wget is required"
        exit 1
    fi

    if [ -z "$VERSION" ]; then
        echo "Error: Could not determine latest version"
        exit 1
    fi
}

# Download and install
install_binary() {
    ARCHIVE_NAME="${BINARY_NAME%-*}-${PLATFORM}.tar.gz"
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/v${VERSION}/${ARCHIVE_NAME}"

    echo "Installing cc-gateway v${VERSION} for ${PLATFORM}..."

    # Create temp directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    # Download
    echo "Downloading from $DOWNLOAD_URL..."
    if command -v curl &> /dev/null; then
        curl -fsSL "$DOWNLOAD_URL" -o "$ARCHIVE_NAME"
    else
        wget -q "$DOWNLOAD_URL" -O "$ARCHIVE_NAME"
    fi

    # Extract
    echo "Extracting..."
    tar -xzf "$ARCHIVE_NAME"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Install binary
    mv "$BINARY_NAME" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    # Cleanup
    cd - > /dev/null
    rm -rf "$TEMP_DIR"

    echo ""
    echo "cc-gateway installed successfully to $INSTALL_DIR/$BINARY_NAME"

    # Check if in PATH
    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo ""
        echo "Warning: $INSTALL_DIR is not in your PATH"
        echo "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
    fi

    # Verify installation
    "$INSTALL_DIR/$BINARY_NAME" --version
}

main() {
    echo "cc-gateway Installer"
    echo "===================="
    echo ""

    detect_platform
    get_latest_version
    install_binary
}

main "$@"
