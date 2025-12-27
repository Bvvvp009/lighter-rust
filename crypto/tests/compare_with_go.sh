#!/bin/bash
# Script to compare Rust and Go hash outputs
# 
# This script runs the Rust test to extract R and message values,
# then runs the Go helper to compute the hash, and compares the results.

set -e

echo "=== Hash Output Comparison: Rust vs Go ==="
echo ""

# Run Rust test and capture output
echo "Running Rust test..."
RUST_OUTPUT=$(cargo test --test verify_hash_computation test_hash_with_r_from_signing -- --nocapture 2>&1)

# Extract R and message values from Rust output
# Look for the "go run" command in the output
GO_CMD=$(echo "$RUST_OUTPUT" | grep -A 1 "To verify with Go" | tail -1 | sed 's/^[[:space:]]*//')

if [ -z "$GO_CMD" ]; then
    echo "Error: Could not extract Go command from Rust output"
    echo "Rust output:"
    echo "$RUST_OUTPUT"
    exit 1
fi

echo "Extracted Go command: $GO_CMD"
echo ""

# Run Go helper
echo "Running Go helper..."
GO_OUTPUT=$(eval "$GO_CMD" 2>&1)

echo "Go output:"
echo "$GO_OUTPUT"
echo ""

# Extract hash from Go output
GO_HASH=$(echo "$GO_OUTPUT" | grep "Hash result (Scalar):" | awk '{print $4}')

# Extract hash from Rust output
RUST_HASH=$(echo "$RUST_OUTPUT" | grep "Computed e'" | grep "Scalar:" | awk '{print $3}')

if [ -z "$GO_HASH" ] || [ -z "$RUST_HASH" ]; then
    echo "Error: Could not extract hash values"
    echo "Go hash: $GO_HASH"
    echo "Rust hash: $RUST_HASH"
    exit 1
fi

echo "=== Comparison ==="
echo "Go hash:   $GO_HASH"
echo "Rust hash: $RUST_HASH"

if [ "$GO_HASH" = "$RUST_HASH" ]; then
    echo ""
    echo "✅ Hashes match!"
    exit 0
else
    echo ""
    echo "❌ Hashes do not match!"
    exit 1
fi







