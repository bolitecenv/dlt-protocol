# DLT Protocol - AI Coding Agent Instructions

## Project Overview
This is a `no_std` Rust library implementing the AUTOSAR Diagnostic Log and Trace (DLT) protocol specification Release 19.11 (with planned R22.11 support). The library is designed for embedded systems without heap allocations, using only stack-based buffers.

## Architecture & Key Components

### Version-Based Module Structure
- `src/r19_11/`: DLT protocol R19.11 implementation (active)
- `src/r22_11/`: DLT protocol R22.11 implementation (placeholder, future)
- Module organization: `lib.rs` exports `r19_11` as the public API

### Core Components (in `src/r19_11/`)
1. **`header.rs`**: Protocol constants and header structures
   - Serial header (`DLT_SERIAL_HEADER_ARRAY`: "DLS\x01")
   - Standard header (4 bytes) + Extra fields (ECU ID, Session ID, Timestamp)
   - Extended header (10 bytes with App ID, Context ID, log level)
   - Bitmask constants: `UEH_MASK`, `WEID_MASK`, `WSID_MASK`, `WTMS_MASK`, etc.

2. **`generate_log.rs`**: Message builder and header generation
   - `DltMessageBuilder`: Main API for creating DLT messages using builder pattern
   - Two key methods:
     - `insert_header_at_front()`: Prepends header to existing payload in buffer
     - `generate_log_message_with_payload()`: Creates complete message with payload copy
   - Endianness handling via `DltEndian` enum (Big/Little)

3. **`payload_headers.rs`**: Verbose mode payload type encoding
   - `PayloadBuilder`: Stack-based builder for typed payloads (strings, ints, floats, raw)
   - Type info encoding follows PRS_Dlt_00626 (4-byte type info + data)
   - `PayloadParser`: Reads and validates typed payload data

4. **`provider.rs`**: Runtime provider pattern for dynamic data
   - `TimestampProvider` and `SessionIdProvider` traits
   - `GlobalProvider<T>`: Thread-safe singleton pattern using `AtomicBool` + `UnsafeCell`
   - Designed for embedded: no heap, const initialization, panic on double-init

5. **`common.rs`**: Shared error types (`DltError::BufferTooSmall`, `InvalidParameter`)

## Critical Patterns & Conventions

### No Heap, Stack-Only Design
- **Never use `Vec`, `Box`, `String`** - everything is `&[u8]` or fixed arrays
- All buffers are caller-provided: `&mut [u8]`
- Use `core::` instead of `std::` (e.g., `core::cmp::min`, `core::sync::atomic`)

### Builder Pattern Usage
```rust
let mut builder = DltMessageBuilder::new()
    .with_ecu_id(b"ECU1")
    .with_app_id(b"APP1")
    .with_context_id(b"CTX1")
    .add_serial_header();

let mut buffer = [0u8; 256];
let payload = b"Log message";
builder.generate_log_message_with_payload(
    &mut buffer, payload, 
    MtinTypeDltLog::DltLogInfo, 1, true
)?;
```

### DLT ID Format
- All IDs are **4 bytes** (`[u8; 4]`), ASCII-encoded
- Use `b"ECU\0"` notation for literals with null padding
- Helper: `to_dlt_id_array()` for converting slices

### Message Counter Management
- Auto-incremented after each message generation
- Access via: `get_counter()`, `reset_counter()`, `increment_counter()`
- Counter wraps at 255 (u8)

### Provider Pattern for Dynamic Values
When timestamp/session ID need runtime sources (e.g., hardware timers):
```rust
static TIMESTAMP_PROVIDER: StaticTimestampProvider = 
    StaticTimestampProvider::new(get_system_timestamp);

fn main() {
    set_global_timestamp_provider(&TIMESTAMP_PROVIDER);
    // Builder will automatically use global provider
}
```

### Buffer Lifetime & Payload Movement
- `insert_header_at_front()` **moves** existing payload backward in the buffer
- Pre-fill payload at buffer start, then call `insert_header_at_front()`
- Returns total size (header + payload) on success

## Testing Patterns

### Integration Tests Location
- `tests/r19_11_it.rs`: Main integration tests (437 lines)
- Test naming: `test_<feature>_<scenario>` (e.g., `test_insert_header_buffer_too_small`)

### Provider Testing Pattern
```rust
static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);
fn get_test_timestamp() -> u32 {
    TEST_COUNTER.fetch_add(1, Ordering::SeqCst)
}
static PROVIDER: StaticTimestampProvider = 
    StaticTimestampProvider::new(get_test_timestamp);
```

### Common Test Assertions
- Verify bitmasks: `buffer[0] & UEH_MASK == UEH_MASK`
- Check IDs at offsets: `&buffer[4..8] == b"ECU1"`
- Validate endianness: `u16::from_le_bytes([buffer[2], buffer[3]])`

## Build & Development

### Build Commands
```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo test               # Run all tests
cargo test r19_11_it     # Run specific test file
```

### No External Dependencies
- `Cargo.toml` has zero dependencies - pure Rust core library
- Edition 2024 - use latest Rust features compatible with no_std

### Adding New Features
1. Add to appropriate module in `src/r19_11/`
2. Export via `mod.rs` using `pub use`
3. Write integration tests in `tests/r19_11_it.rs`
4. Verify no std usage with `#![no_std]` attribute at module top

## Key Gotchas

1. **Buffer size calculation**: Always include serial header (4 bytes) if enabled
   - Standard header: 4 bytes
   - Extra fields: 4 (ECU) + 4 (Session) + 4 (Timestamp) = 12 bytes
   - Extended header: 10 bytes
   - Total minimum: 26 bytes (30 with serial header)

2. **Verbose bit**: Bit 0 of `msin` (extended header byte 0)
   - `true` = payload has type info headers
   - `false` = non-verbose (raw binary payload)

3. **Type Info Encoding**: Little-endian 32-bit field
   - Bits 0-3: `TypeLength` (size class)
   - Bits 4-15: `PayloadType` flags (one-hot encoded)

4. **Global Provider**: Can only be set **once** - panics on re-initialization
   - Use different static instances for different test cases
   - Not suitable for dynamic reconfiguration

## Incomplete/Future Work
- `src/r19_11/payload.rs`: Empty placeholder
- `src/r19_11/generate_service.rs`: Empty placeholder  
- `src/r22_11/`: Planned future DLT R22.11 implementation
- Parser/decoder functionality (only encoder currently implemented)
