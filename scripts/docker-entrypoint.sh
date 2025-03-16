#!/bin/bash
set -e

echo "Starting MyHealthGuide API in development mode..."

# Check if the Cargo.lock file exists and if it's the correct version
if [ -f "/app/Cargo.lock" ]; then
    echo "Found existing Cargo.lock file. Checking compatibility..."
    
    # Check if Cargo can read the lock file
    if ! cargo check --dry-run &> /dev/null; then
        echo "Cargo.lock is incompatible with the current Rust version. Regenerating..."
        # Force to ignore the existing Cargo.lock by temporarily renaming it
        mv /app/Cargo.lock /app/Cargo.lock.backup
        
        # Generate a new Cargo.lock
        cargo generate-lockfile
        
        # Inform the user
        echo "Generated a new Cargo.lock file compatible with Rust $(rustc --version)"
    else
        echo "Cargo.lock is compatible."
    fi
else
    echo "No Cargo.lock found. Generating one..."
    cargo generate-lockfile
fi

# Run the application
echo "Starting application..."
cargo run --bin MyHealthGuide-api

# Keep container running if the command fails
if [ $? -ne 0 ]; then
    echo "Application crashed. Container will keep running for debugging."
    tail -f /dev/null
fi 