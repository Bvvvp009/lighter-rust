# Build Rust WASM signer for TypeScript SDK
# 
# Usage:
#   just build-wasm          # Build WASM for web target (browser)
#   just build-wasm-nodejs   # Build WASM for Node.js backend
#   just build-wasm-all      # Build both web and Node.js targets
#   just check-wasm          # Check WASM compilation
#   just install-wasm-pack   # Install wasm-pack if missing

# Build WASM for web target (browser)
build-wasm:
    #!/usr/bin/env bash
    @echo "Building Rust WASM signer for web (browser)..."
    @cd signer-wasm && wasm-pack build --target web --out-dir ../wasm-web --release
    @echo "Build complete! WASM files are in rust-signer/wasm-web/"

# Build WASM for Node.js backend (no wasm_exec.js needed)
build-wasm-nodejs:
    #!/usr/bin/env bash
    @echo "Building Rust WASM signer for Node.js backend..."
    @cd signer-wasm && wasm-pack build --target nodejs --out-dir ../wasm-nodejs --release
    @echo "Build complete! WASM files are in rust-signer/wasm-nodejs/"

# Build both web and Node.js targets
build-wasm-all: build-wasm build-wasm-nodejs
    @echo "All WASM builds complete!"
    @echo "  - Browser: rust-signer/wasm-web/"
    @echo "  - Node.js: rust-signer/wasm-nodejs/"

# Check WASM compilation (without building)
check-wasm:
    cargo check --manifest-path signer-wasm/Cargo.toml

# Install wasm-pack if not already installed
install-wasm-pack:
    #!/usr/bin/env bash
    @if ! command -v wasm-pack &> /dev/null; then
        echo "Installing wasm-pack..."
        cargo install wasm-pack
    else
        echo "wasm-pack is already installed"
    fi

# Clean WASM build artifacts
clean-wasm:
    @echo "Cleaning WASM build artifacts..."
    @rm -rf wasm-web/ wasm-nodejs/
    @cd signer-wasm && cargo clean
    @echo "Clean complete!"

# Full build with installation check (web only)
build-wasm-full: install-wasm-pack build-wasm

# Full build with installation check (all targets)
build-wasm-all-full: install-wasm-pack build-wasm-all

# Benchmark WASM performance (comprehensive with real API calls)
benchmark-wasm:
    @cd benchmark && node -r ts-node/register benchmark-comprehensive.ts

benchmark-wasm-orders ORDERS:
    @cd benchmark && node -r ts-node/register benchmark-comprehensive.ts {{ORDERS}}

# Show help
default:
    @echo "Available commands:"
    @echo "  just build-wasm            - Build WASM for web target (browser)"
    @echo "  just build-wasm-nodejs     - Build WASM for Node.js backend"
    @echo "  just build-wasm-all        - Build both web and Node.js targets"
    @echo "  just check-wasm            - Check WASM compilation"
    @echo "  just install-wasm-pack     - Install wasm-pack if missing"
    @echo "  just clean-wasm            - Clean WASM build artifacts"
    @echo "  just build-wasm-full       - Install wasm-pack and build (web)"
    @echo "  just build-wasm-all-full   - Install wasm-pack and build (all)"
    @echo "  just benchmark-wasm        - Compare Rust vs Go WASM performance"
    @echo "  just benchmark-wasm-orders N - Benchmark with N orders"

