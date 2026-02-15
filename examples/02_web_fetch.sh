#!/bin/bash

# Example 2: Web Fetching & Summarization
# This example demonstrates Zene's ability to fetch content from the web using 'fetch_url' 
# and process it using its reasoning capabilities.

echo "Running Example 2: Web Fetch & Summarize..."

# Ensure we are in the project root
cd "$(dirname "$0")/.."

# Target URL (Example.com is stable and small)
URL="https://example.com"

# Run Zene
# Note: We ask it to save the result to a file so we can verify it.
cargo run -- run "Fetch the content of $URL. Then, create a file named 'web_summary.md' that contains a short summary of what the page is about."

echo "---------------------------------------------------"
if [ -f "web_summary.md" ]; then
    echo "Success! Summary created:"
    cat web_summary.md
    # Cleanup
    rm web_summary.md
else
    echo "Failed: Summary file was not created."
fi
