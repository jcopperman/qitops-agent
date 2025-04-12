#!/bin/bash
# QitOps Agent Installation Script for Linux/macOS
# This script builds the QitOps Agent and installs it to a location in the PATH

set -e

# Build the release version
echo -e "\033[0;36mBuilding QitOps Agent...\033[0m"
cargo build --release

# Create the installation directory if it doesn't exist
INSTALL_DIR="$HOME/.qitops/bin"
echo -e "\033[0;36mCreating installation directory: $INSTALL_DIR\033[0m"
mkdir -p "$INSTALL_DIR"

# Copy the binary to the installation directory
echo -e "\033[0;36mInstalling QitOps Agent to $INSTALL_DIR\033[0m"
cp "target/release/qitops" "$INSTALL_DIR/qitops"
chmod +x "$INSTALL_DIR/qitops"

# Check if the installation directory is in the PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "\033[0;36mAdding installation directory to PATH\033[0m"
    
    # Determine shell configuration file
    SHELL_CONFIG=""
    if [[ "$SHELL" == *"zsh"* ]]; then
        SHELL_CONFIG="$HOME/.zshrc"
    elif [[ "$SHELL" == *"bash"* ]]; then
        if [[ -f "$HOME/.bashrc" ]]; then
            SHELL_CONFIG="$HOME/.bashrc"
        elif [[ -f "$HOME/.bash_profile" ]]; then
            SHELL_CONFIG="$HOME/.bash_profile"
        fi
    fi
    
    if [[ -n "$SHELL_CONFIG" ]]; then
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$SHELL_CONFIG"
        echo -e "\033[0;32mAdded $INSTALL_DIR to PATH in $SHELL_CONFIG\033[0m"
        echo -e "\033[0;33mPlease restart your terminal or run 'source $SHELL_CONFIG' to update your PATH\033[0m"
    else
        echo -e "\033[0;33mCould not determine shell configuration file. Please add the following line to your shell configuration file:\033[0m"
        echo "export PATH=\"\$PATH:$INSTALL_DIR\""
    fi
fi

echo -e "\033[0;32mQitOps Agent has been installed successfully!\033[0m"
echo -e "\033[0;32mYou can now use the 'qitops' command from any terminal.\033[0m"
echo -e "\033[0;36mTry 'qitops --help' to get started.\033[0m"
