#!/usr/bin/env bash

set -e

BINARY_DIR="$HOME/.local/bin"
STDLIB_DIR="$HOME/.local/share/tol"

echo "Building tol-lang release binary..."
cargo build --release

echo "Installing binary to $BINARY_DIR..."
mkdir -p "$BINARY_DIR"
cp ./target/release/tol-lang "$BINARY_DIR/tol"

echo "Installing standard library to $STDLIB_DIR..."
mkdir -p "$STDLIB_DIR"
cp -r stdlib "$STDLIB_DIR"

EXPORT_LINE="export TOL_STDLIB=$STDLIB_DIR/stdlib"

# Add TOL_STDLIB export to shell profile if not already present
add_to_profile() {
    local profile_file="$1"
    if [ -f "$profile_file" ]; then
        if ! grep -q "TOL_STDLIB" "$profile_file"; then
            echo "$EXPORT_LINE" >> "$profile_file"
            echo "Added TOL_STDLIB to $profile_file"
        fi
    fi
}

add_to_profile "$HOME/.zshrc"
add_to_profile "$HOME/.bashrc"
add_to_profile "$HOME/.bash_profile"

echo "Installation complete! Make sure $BINARY_DIR is in your PATH."
