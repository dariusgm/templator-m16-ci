#!/bin/sh
# Build script for {{ crate_name }}

echo "Building {{ crate_name }}..."
cargo build --release
echo "✓ Build complete"
