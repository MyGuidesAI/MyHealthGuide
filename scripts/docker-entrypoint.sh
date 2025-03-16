#!/bin/bash
set -e

echo "Starting MyHealthGuide API in development mode..."

# Check if Cargo.lock exists and is compatible with the current Rust version
if [ -f /app/Cargo.lock ]; then
    echo "Found existing Cargo.lock file. Checking compatibility..."
    if ! cargo --version | grep -q "$(grep -A 1 '\[metadata\]' /app/Cargo.lock | grep -oP 'rustc \K[0-9]+\.[0-9]+\.[0-9]+')"; then
        echo "Cargo.lock is incompatible with the current Rust version. Regenerating..."
        mv /app/Cargo.lock /app/Cargo.lock.old
        cargo update
        echo "Generated a new Cargo.lock file compatible with Rust $(rustc --version)"
    fi
else
    echo "No Cargo.lock file found. Generating..."
    cargo update
    echo "Generated a new Cargo.lock file compatible with Rust $(rustc --version)"
fi

echo "Starting application..."
# Run the application
cargo run --bin my_health_guide_api

# Keep the container running if the application crashes
echo "Application crashed. Keeping container running for debugging..."
tail -f /dev/null
