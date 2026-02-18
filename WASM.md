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
- Supports optional DLT file header (`DLT\x01`) for file-based message storage

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

## WASM Demo Examples

Build and run the WASM demo pages:

```bash
# Build WASM example
bash build-wasm.sh

# Start a local server
python3 -m http.server 8000
```

Available demo pages at `http://localhost:8000/examples/`:

| Page | Description |
|------|-------------|
| `wasm_test.html` | Basic message create/analyze, with file header support |
| `test_messages.html` | Multi-message hex parser with file header detection |
| `wasm_service_test.html` | Log & service message generator/parser with file header option |
| `simple_demo.html` | Japanese-language demo with file header generation |
| `dlt_stream.html` | Real-time WebSocket stream viewer with file header detection |

### DLT File Header Support

The WASM API supports the optional 4-byte DLT file header (`DLT\x01` / `0x44 0x4C 0x54 0x01`), which is used when storing DLT messages to files. Key functions:

- **`create_dlt_message_with_file_header()`** — generates a message with file header prepended
- **`get_has_file_header(buffer_ptr, buffer_len)`** — returns 1 if buffer starts with `DLT\x01`
- **`generate_log_message()` config byte 15** — set to 1 to prepend file header to generated log messages
- **`analyze_dlt_message()`** — automatically detects and skips file header during parsing
