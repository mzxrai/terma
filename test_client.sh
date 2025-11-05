#!/bin/bash
# Simple test script for the Terma client

set -e

echo "Building client..."
cargo build --release --bin terma-client

echo ""
echo "Test instructions:"
echo "1. Open two terminal windows"
echo "2. In both, run: ./target/release/terma-client localhost:3000 test123"
echo "3. Type messages in one and see them appear in the other"
echo "4. Press Ctrl+C to quit"
echo ""
echo "Or create a room via web:"
echo "1. Visit http://localhost:3000"
echo "2. Click 'Create New Room'"
echo "3. Copy the command and run it in your terminal"
