#!/usr/bin/env bash
# lin installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/boyeln/lin/main/install.sh | bash
#
# This script detects your OS and architecture, downloads the appropriate
# binary from GitHub releases, and installs it to ~/.local/bin

set -euo pipefail

# Configuration
REPO="boyeln/lin"
BINARY_NAME="lin"
INSTALL_DIR="${LIN_INSTALL_DIR:-$HOME/.local/bin}"

# Colors (disabled if not a terminal)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    BOLD=''
    NC=''
fi

info() {
    echo -e "${BLUE}info${NC}: $1"
}

success() {
    echo -e "${GREEN}success${NC}: $1"
}

warn() {
    echo -e "${YELLOW}warn${NC}: $1"
}

error() {
    echo -e "${RED}error${NC}: $1" >&2
    exit 1
}

# Detect OS
detect_os() {
    local os
    os="$(uname -s)"
    case "$os" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *)       error "Unsupported operating system: $os" ;;
    esac
}

# Detect architecture
detect_arch() {
    local arch
    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64)  echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *)             error "Unsupported architecture: $arch" ;;
    esac
}

# Get the download URL for the appropriate binary
get_download_url() {
    local os="$1"
    local arch="$2"
    local version="$3"

    local target
    case "$os" in
        linux)
            target="${arch}-unknown-linux-gnu"
            ;;
        macos)
            target="${arch}-apple-darwin"
            ;;
        windows)
            target="${arch}-pc-windows-msvc"
            ;;
    esac

    local ext="tar.gz"
    if [ "$os" = "windows" ]; then
        ext="zip"
    fi

    echo "https://github.com/${REPO}/releases/download/${version}/${BINARY_NAME}-${version}-${target}.${ext}"
}

# Get the latest release version from GitHub
get_latest_version() {
    local url="https://api.github.com/repos/${REPO}/releases/latest"

    if command -v curl &> /dev/null; then
        curl -fsSL "$url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    elif command -v wget &> /dev/null; then
        wget -qO- "$url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
}

# Download a file
download() {
    local url="$1"
    local output="$2"

    if command -v curl &> /dev/null; then
        curl -fsSL "$url" -o "$output"
    elif command -v wget &> /dev/null; then
        wget -q "$url" -O "$output"
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
}

# Check if lin is already installed and get its version
get_installed_version() {
    if command -v "$BINARY_NAME" &> /dev/null; then
        "$BINARY_NAME" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo ""
    else
        echo ""
    fi
}

# Main installation function
main() {
    echo -e "${BOLD}lin installer${NC}"
    echo ""

    # Detect platform
    local os arch
    os="$(detect_os)"
    arch="$(detect_arch)"
    info "Detected platform: ${os}/${arch}"

    # Check for existing installation
    local installed_version
    installed_version="$(get_installed_version)"
    if [ -n "$installed_version" ]; then
        info "Found existing installation: v${installed_version}"
    fi

    # Get latest version
    info "Fetching latest release..."
    local version
    version="$(get_latest_version)"
    if [ -z "$version" ]; then
        error "Could not determine latest version. Check your internet connection or visit https://github.com/${REPO}/releases"
    fi
    info "Latest version: ${version}"

    # Check if already up to date
    local version_number="${version#v}"  # Remove 'v' prefix if present
    if [ "$installed_version" = "$version_number" ]; then
        success "Already up to date (v${installed_version})"
        exit 0
    fi

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Create temp directory for download
    local tmp_dir
    tmp_dir="$(mktemp -d)"
    trap 'rm -rf "$tmp_dir"' EXIT

    # Download
    local download_url
    download_url="$(get_download_url "$os" "$arch" "$version")"
    info "Downloading from: ${download_url}"

    local archive_name="lin-archive"
    if [ "$os" = "windows" ]; then
        archive_name="${archive_name}.zip"
    else
        archive_name="${archive_name}.tar.gz"
    fi

    download "$download_url" "${tmp_dir}/${archive_name}"

    # Extract
    info "Extracting..."
    cd "$tmp_dir"
    if [ "$os" = "windows" ]; then
        unzip -q "$archive_name"
    else
        tar xzf "$archive_name"
    fi

    # Install
    local binary_path="${tmp_dir}/${BINARY_NAME}"
    if [ "$os" = "windows" ]; then
        binary_path="${binary_path}.exe"
    fi

    if [ ! -f "$binary_path" ]; then
        error "Binary not found in archive"
    fi

    chmod +x "$binary_path"
    mv "$binary_path" "${INSTALL_DIR}/${BINARY_NAME}"

    if [ -n "$installed_version" ]; then
        success "Upgraded lin from v${installed_version} to ${version}"
    else
        success "Installed lin ${version} to ${INSTALL_DIR}/${BINARY_NAME}"
    fi

    # Check if install directory is in PATH
    case ":$PATH:" in
        *":${INSTALL_DIR}:"*)
            # Already in PATH
            ;;
        *)
            echo ""
            warn "${INSTALL_DIR} is not in your PATH"
            echo ""
            echo "Add it to your shell configuration:"
            echo ""
            echo "  # For bash (~/.bashrc)"
            echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
            echo ""
            echo "  # For zsh (~/.zshrc)"
            echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
            echo ""
            echo "  # For fish (~/.config/fish/config.fish)"
            echo "  fish_add_path ~/.local/bin"
            echo ""
            ;;
    esac

    # Verify installation
    if command -v "$BINARY_NAME" &> /dev/null; then
        echo ""
        info "Run 'lin --help' to get started"
    else
        echo ""
        info "Run '${INSTALL_DIR}/${BINARY_NAME} --help' to get started"
    fi
}

main "$@"
