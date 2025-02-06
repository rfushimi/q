#!/bin/bash

set -e

# Default installation directory
DEFAULT_BIN_DIR="$HOME/.bin"
BIN_DIR="${BIN_DIR:-$DEFAULT_BIN_DIR}"
GITHUB_REPO="ryohei/q"
BINARY_NAME="q"
TEMP_DIR="/tmp/q-install"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Check for required commands
check_command() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${RED}Error: $1 is required but not installed.${NC}"
        exit 1
    fi
}

check_command "git"
check_command "cargo"
check_command "rustc"

# Create installation directory if it doesn't exist
mkdir -p "$BIN_DIR"

echo -e "${BLUE}Installing $BINARY_NAME...${NC}"

# Clean up any existing temporary directory
rm -rf "$TEMP_DIR"
mkdir -p "$TEMP_DIR"
cd "$TEMP_DIR"

# Clone the repository
echo -e "${BLUE}Cloning repository...${NC}"
git clone "https://github.com/$GITHUB_REPO.git" .

# Build the project
echo -e "${BLUE}Building project...${NC}"
cargo build --release

# Install the binary
echo -e "${BLUE}Installing binary...${NC}"
cp "target/release/$BINARY_NAME" "$BIN_DIR/"

# Clean up
cd
rm -rf "$TEMP_DIR"

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

# Print Rust version information
echo -e "\n${BLUE}Installation details:${NC}"
echo -e "Rust version: $(rustc --version)"
echo -e "Cargo version: $(cargo --version)"
echo -e "Installation directory: $BIN_DIR"
