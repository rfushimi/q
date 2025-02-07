#!/bin/bash

# Print Homebrew installation instructions
if command -v brew &> /dev/null; then
    echo -e "${BLUE}Homebrew detected!${NC}"
    echo -e "${BLUE}You can install q using Homebrew with:${NC}"
    echo -e "${GREEN}brew tap rfushimi/tap${NC}"
    echo -e "${GREEN}brew install q${NC}"
    echo -e "${BLUE}Or continue with manual installation...${NC}"
    echo
fi

set -e

# Default installation directory
DEFAULT_BIN_DIR="$HOME/.bin"
BIN_DIR="${BIN_DIR:-$DEFAULT_BIN_DIR}"
GITHUB_REPO="rfushimi/q"
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
check_command "curl"

# Install Rust if not installed
if ! command -v rustc &> /dev/null; then
    echo -e "${BLUE}Installing Rust...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

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

# Install dependencies and build the project
echo -e "${BLUE}Installing dependencies and building project...${NC}"
# Source cargo environment in case it was just installed
source "$HOME/.cargo/env"
# Set release mode environment variables
export RUSTFLAGS="-C target-cpu=native -C opt-level=3"
export CARGO_PROFILE_RELEASE_DEBUG=0
export CARGO_PROFILE_RELEASE_LTO=true
# Update dependencies
cargo update
# Build the project in release mode with optimizations
cargo build --release --quiet

if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed. Please check the error messages above.${NC}"
    exit 1
fi

# Install the binary
echo -e "${BLUE}Installing binary...${NC}"
cp "target/release/$BINARY_NAME" "$BIN_DIR/"
strip "$BIN_DIR/$BINARY_NAME"

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

# Print version information
echo -e "\n${BLUE}Installation details:${NC}"
echo -e "Rust version: $(rustc --version)"
echo -e "Cargo version: $(cargo --version)"
echo -e "Installation directory: $BIN_DIR"
