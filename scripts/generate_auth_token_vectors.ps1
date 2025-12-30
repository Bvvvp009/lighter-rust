# PowerShell script to generate auth token test vectors from Go implementation

Write-Host "Generating auth token test vectors from Go..." -ForegroundColor Green

# Change to lighter-go directory
Push-Location lighter-go

try {
    # Run the Go test to generate test vectors
    Write-Host "Running Go test..." -ForegroundColor Yellow
    go test -v ./signer -run TestGenerateAuthTokenTestVectors 2>&1 | Tee-Object -FilePath "$env:TEMP\go_auth_token_output.txt"
    
    Write-Host ""
    Write-Host "Test vectors generated. Output saved to $env:TEMP\go_auth_token_output.txt" -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "1. Review the output to extract test vectors"
    Write-Host "2. Update lighter-rust/signer/tests/auth_token_comparison.rs with the test vectors"
    Write-Host "3. Run: cargo test --test auth_token_comparison test_auth_token_matches_go"
}
finally {
    Pop-Location
}

















