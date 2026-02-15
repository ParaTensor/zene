#!/bin/bash

# Example 1: Basic File Manipulation
# This example demonstrates Zene's ability to create files using the 'write_file' tool.

echo "Running Example 1: Hello World File Creation..."

# Ensure we are in the project root
cd "$(dirname "$0")/.."

# Run Zene
cargo run -- run "Please create a file named 'hello_zene.txt' in the current directory with the content: 'Hello from Zene! This file was created by an AI agent.'"

echo "---------------------------------------------------"
if [ -f "hello_zene.txt" ]; then
    echo "Success! File created:"
    cat hello_zene.txt
    # Cleanup
    rm hello_zene.txt
else
    echo "Failed: File was not created."
fi
