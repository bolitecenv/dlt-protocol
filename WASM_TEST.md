# WASM Test Guide

## Quick Start

Run the WASM test in your browser:

```bash
# 1. Build WASM example
./build-wasm.sh

# 2. Start local server
python3 -m http.server 8000

# 3. Open in browser
open http://localhost:8000/examples/wasm_test.html
```

## What's Included

### Files
- **[examples/wasm_demo.rs](examples/wasm_demo.rs)**: WASM example with exported functions
- **[examples/wasm_test.html](examples/wasm_test.html)**: Interactive test page
- **[build-wasm.sh](build-wasm.sh)**: Build script

### Exported Functions
- `create_dlt_message(buffer_ptr, buffer_len)`: Creates a DLT log message
- `get_version()`: Returns library version
- `allocate(size)`: Allocates memory in WASM
- `reset_allocator()`: Resets memory allocator

### Test Features
✅ Creates DLT messages with ECU ID, App ID, Context ID  
✅ Displays message in hex format  
✅ Shows message size and structure  
✅ Interactive browser UI  

## Manual Build

```bash
cargo build --target wasm32-unknown-unknown --example wasm_demo --release
```

The output will be at:
```
target/wasm32-unknown-unknown/release/examples/wasm_demo.wasm
```

## File Size
~6.6 KB (optimized release build)
