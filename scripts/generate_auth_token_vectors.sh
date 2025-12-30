#!/bin/bash
# Script to generate auth token test vectors from Go implementation
# and update Rust test file

set -e

echo "Generating auth token test vectors from Go..."

# Change to lighter-go directory
cd lighter-go

# Run the Go test to generate test vectors
echo "Running Go test..."
go test -v ./signer -run TestGenerateAuthTokenTestVectors 2>&1 | tee /tmp/go_auth_token_output.txt

echo ""
echo "Test vectors generated. Output saved to /tmp/go_auth_token_output.txt"
echo ""
echo "Next steps:"
echo "1. Review the output to extract test vectors"
echo "2. Update lighter-rust/signer/tests/auth_token_comparison.rs with the test vectors"
echo "3. Run: cargo test --test auth_token_comparison test_auth_token_matches_go"

















