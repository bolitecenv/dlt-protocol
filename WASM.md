# WebAssembly (WASM) Support

This library fully supports compilation to WebAssembly targets.

## Building for WASM

### Prerequisites
Ensure you have the WASM target installed:
```bash
rustup target add wasm32-unknown-unknown
```

### Build Commands

**Debug build:**
```bash
cargo build --target wasm32-unknown-unknown --lib
```

**Release build (optimized):**
```bash
cargo build --target wasm32-unknown-unknown --lib --release
```

### Output Location
The compiled WASM binary will be located at:
- Debug: `target/wasm32-unknown-unknown/debug/dlt_protocol.wasm`
- Release: `target/wasm32-unknown-unknown/release/dlt_protocol.wasm`

## Architecture

The library is designed as a `no_std` Rust library that:
- Uses only stack-based buffers (no heap allocations)
- Works in WASM environments without requiring std library
- Includes a simple panic handler for WASM targets
- Maintains full compatibility with native builds and tests

### Build Configuration

The library uses conditional compilation:
- **WASM targets**: Builds with `#![no_std]` and includes a panic handler
- **Native builds**: Uses standard library for full functionality
- **Tests**: Always use standard library for test infrastructure

### Features

- `std` feature (optional): Explicitly enable standard library support
  ```bash
  cargo build --target wasm32-unknown-unknown --features std
  ```

## Usage in Web Applications

The WASM binary can be integrated into JavaScript/TypeScript applications using tools like:
- `wasm-pack` for npm package creation
- `wasm-bindgen` for JavaScript bindings
- Direct WASM module loading in browsers

## Testing

Regular Rust tests run against the native build:
```bash
cargo test
```

All 57 integration tests pass successfully while maintaining WASM compatibility.
