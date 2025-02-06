#!/bin/bash

set -e

# Default installation directory
DEFAULT_BIN_DIR="$HOME/.bin"
BIN_DIR="${BIN_DIR:-$DEFAULT_BIN_DIR}"
GITHUB_REPO="ryohei/q"
BINARY_NAME="q"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Create installation directory if it doesn't exist
mkdir -p "$BIN_DIR"

echo -e "${BLUE}Installing $BINARY_NAME...${NC}"

# Detect operating system and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case $OS in
    Linux)
        OS="linux"
        ;;
    Darwin)
        OS="darwin"
        ;;
    *)
        echo -e "${RED}Error: Unsupported operating system: $OS${NC}"
        exit 1
        ;;
esac

case $ARCH in
    x86_64)
        ARCH="x86_64"
        ;;
    arm64|aarch64)
        ARCH="aarch64"
        ;;
    *)
        echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

# Get the latest release version
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_RELEASE" ]; then
    echo -e "${RED}Error: Could not determine the latest release version${NC}"
    exit 1
fi

# Download URL
DOWNLOAD_URL="https://github.com/$GITHUB_REPO/releases/download/$LATEST_RELEASE/$BINARY_NAME-$OS-$ARCH.tar.gz"

# Create temporary directory
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

echo "Downloading $BINARY_NAME $LATEST_RELEASE..."
if ! curl -sL "$DOWNLOAD_URL" -o "$BINARY_NAME.tar.gz"; then
    echo -e "${RED}Error: Failed to download $BINARY_NAME${NC}"
    exit 1
fi

# Extract the binary
tar xzf "$BINARY_NAME.tar.gz"

# Make it executable
chmod +x "$BINARY_NAME"

# Move to installation directory
mv "$BINARY_NAME" "$BIN_DIR/"

# Clean up
cd
rm -rf "$TMP_DIR"

echo -e "${GREEN}Successfully installed $BINARY_NAME to $BIN_DIR/$BINARY_NAME${NC}"

# Add to PATH if not already there
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo -e "${BLUE}Adding $BIN_DIR to PATH...${NC}"
    
    # Detect shell
    SHELL_NAME=$(basename "$SHELL")
    SHELL_CONFIG=""
    
    case $SHELL_NAME in
        bash)
            SHELL_CONFIG="$HOME/.bashrc"
            ;;
        zsh)
            SHELL_CONFIG="$HOME/.zshrc"
            ;;
        *)
            echo -e "${RED}Warning: Unsupported shell: $SHELL_NAME${NC}"
            echo -e "${RED}Please manually add $BIN_DIR to your PATH${NC}"
            exit 0
            ;;
    esac
    
    echo "export PATH=\"\$PATH:$BIN_DIR\"" >> "$SHELL_CONFIG"
    echo -e "${GREEN}Added $BIN_DIR to PATH in $SHELL_CONFIG${NC}"
    echo -e "${BLUE}Please restart your shell or run: source $SHELL_CONFIG${NC}"
fi

echo -e "${GREEN}Installation complete!${NC}"
echo -e "${BLUE}Run 'q --help' to get started${NC}"
