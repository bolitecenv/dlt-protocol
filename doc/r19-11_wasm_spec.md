# DLT R19.11 WASM API Specification

This document is the authoritative reference for AI agents and developers integrating the
`wasm_demo.wasm` module. It covers every exported function, all memory layouts, error
codes, and complete usage examples in JavaScript and Node.js.

---

## Table of Contents

1. [File Location & Build](#1-file-location--build)
2. [Loading the Module](#2-loading-the-module)
3. [Memory Model](#3-memory-model)
4. [Error Codes](#4-error-codes)
5. [Allocator API](#5-allocator-api)
6. [Message Analysis](#6-message-analysis)
7. [Message Generation — Log](#7-message-generation--log)
8. [Message Generation — Service](#8-message-generation--service)
9. [Service Message Parsing](#9-service-message-parsing)
10. [Payload Formatting](#10-payload-formatting)
11. [Utilities](#11-utilities)
12. [DLT Message Structure Reference](#12-dlt-message-structure-reference)
13. [Constants Reference](#13-constants-reference)
14. [Complete Node.js Example](#14-complete-nodejs-example)
15. [Complete Browser Example](#15-complete-browser-example)

---

## 1. File Location & Build

| Item | Value |
|------|-------|
| WASM source | `examples/wasm_demo.rs` |
| Built artifact | `target/wasm32-unknown-unknown/release/examples/wasm_demo.wasm` |
| Build command | `./build-wasm.sh` |
| Rust edition | 2024 |
| Target | `wasm32-unknown-unknown` |

The module has **no JavaScript glue layer** (no `wasm-pack`). All interaction is via
the raw WebAssembly C ABI using pointer arithmetic and `WebAssembly.Memory`.

---

## 2. Loading the Module

### Node.js (ESM)

```js
import { readFileSync } from 'fs';

const wasmBytes = readFileSync('target/wasm32-unknown-unknown/release/examples/wasm_demo.wasm');
const { instance } = await WebAssembly.instantiate(wasmBytes, {});
const wasm = instance.exports;
```

### Browser

```js
const { instance } = await WebAssembly.instantiateStreaming(
  fetch('wasm_demo.wasm'), {}
);
const wasm = instance.exports;
```

### Required imports

The module imports **nothing** — pass an empty `{}` imports object.

---

## 3. Memory Model

The WASM module contains:

- **`WebAssembly.Memory`** — linear memory (exported as `memory`)
- **Internal heap** — 16 384-byte static arena managed by `allocate` / `deallocate`

All pointers are byte offsets into `memory.buffer` (a `SharedArrayBuffer` or
`ArrayBuffer`).

```js
const mem  = wasm.memory;          // WebAssembly.Memory
const view = new Uint8Array(mem.buffer);   // byte view
const dv   = new DataView(mem.buffer);     // typed multi-byte reads
```

> **Rule**: After any call that can trigger an allocation (e.g. `analyze_dlt_message`),
> re-create your `DataView` / `Uint8Array` wrappers because the underlying `ArrayBuffer`
> may have been replaced if memory grew.

---

## 4. Error Codes

All functions that return `i32` use these negative values to signal errors.

| Code | Value | Meaning |
|------|-------|---------|
| `ERROR_NULL_POINTER` | `-1` | A required pointer argument was null |
| `ERROR_BUFFER_TOO_SMALL` | `-2` | Provided buffer is too small |
| `ERROR_INVALID_FORMAT` | `-3` | Data cannot be parsed / invalid parameters |
| `ERROR_OUT_OF_MEMORY` | `-4` | Internal heap exhausted |

A return value **≥ 1** is always a byte count (success).

---

## 5. Allocator API

The module exposes a bump allocator. Use it to obtain WASM memory for input/output
buffers without managing raw offsets yourself.

### `allocate(size: i32) → i32`

Allocate `size` bytes. Returns a pointer (> 0) or 0 on failure.

```js
const ptr = wasm.allocate(256);
if (!ptr) throw new Error('WASM heap full');
```

### `deallocate(ptr: i32)`

Mark an allocation as free. The bump allocator does not compact — call
`reset_allocator()` to reclaim all memory at once.

```js
wasm.deallocate(ptr);
```

### `reset_allocator()`

Reset the entire heap to empty (zeros all memory). Invalidates all previously
returned pointers.

```js
wasm.reset_allocator();
```

### `get_heap_usage() → i32`

Returns bytes currently consumed from the internal heap.

### `get_heap_capacity() → i32`

Returns total internal heap capacity (16 384).

**Pattern — allocate, use, free:**

```js
const ptr = wasm.allocate(256);
new Uint8Array(wasm.memory.buffer, ptr, 256).set(myBytes);
// ... call wasm functions ...
wasm.deallocate(ptr);
```

---

## 6. Message Analysis

### `analyze_dlt_message(buffer_ptr: i32, buffer_len: i32) → i32`

Parse a complete DLT message (with or without file/serial headers) and return a
pointer to a 32-byte `AnalysisResult` struct.  
Returns `0` (null) on parse failure.

**Input:** Any valid DLT byte sequence (file header + serial header + standard header
+ optional extra fields + extended header + payload). Incomplete messages return null.

**Output struct layout (32 bytes, all integers little-endian):**

| Offset | Size | Type | Field | Description |
|--------|------|------|-------|-------------|
| 0 | 2 | u16 LE | `total_len` | Total standard-header `len` field (excl. serial/file prefixes) |
| 2 | 2 | u16 LE | `header_len` | Byte length of all header fields (standard + extra + extended) |
| 4 | 2 | u16 LE | `payload_len` | Byte length of the payload section |
| 6 | 2 | u16 LE | `payload_offset` | Offset of payload start from byte 0 of the input buffer (accounts for full 16-byte storage header if present) |
| 8 | 1 | u8 | `msg_type` | Raw MSIN byte from extended header |
| 9 | 1 | u8 | `log_level` | Numeric log level (0=none, 1=Fatal … 6=Verbose) |
| 10 | 1 | u8 | `has_serial` | 1 if serial header `DLS\x01` was present |
| 11 | 1 | u8 | `has_ecu` | 1 if ECU ID field is present |
| 12 | 4 | u8[4] | `ecu_id` | ECU ID bytes (ASCII, null padded) |
| 16 | 4 | u8[4] | `app_id` | Application ID bytes |
| 20 | 4 | u8[4] | `ctx_id` | Context ID bytes |
| 24 | 1 | u8 | `mstp` | Message Type (0=Log, 1=Trace, 2=Network, 3=Control) |
| 25 | 1 | u8 | `is_verbose` | 1 if verbose payload flag is set |
| 26 | 6 | u8[6] | reserved | Zeroed, reserved for future use |

**Caller must deallocate the returned pointer.**

```js
function analyzeMessage(wasm, bytes) {
  const ptr = wasm.allocate(bytes.length);
  new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);

  const resultPtr = wasm.analyze_dlt_message(ptr, bytes.length);
  wasm.deallocate(ptr);
  if (!resultPtr) return null;

  const dv = new DataView(wasm.memory.buffer, resultPtr, 32);
  const result = {
    totalLen:      dv.getUint16(0, true),
    headerLen:     dv.getUint16(2, true),
    payloadLen:    dv.getUint16(4, true),
    payloadOffset: dv.getUint16(6, true),
    msgType:       dv.getUint8(8),
    logLevel:      dv.getUint8(9),
    hasSerial:     dv.getUint8(10),
    hasEcu:        dv.getUint8(11),
    ecuId:  id4(wasm, resultPtr + 12),
    appId:  id4(wasm, resultPtr + 16),
    ctxId:  id4(wasm, resultPtr + 20),
    mstp:          dv.getUint8(24),
    isVerbose:     dv.getUint8(25),
  };
  wasm.deallocate(resultPtr);
  return result;
}

// Helper: read 4 ASCII bytes as a trimmed string
function id4(wasm, offset) {
  return String.fromCharCode(
    ...new Uint8Array(wasm.memory.buffer, offset, 4)
  ).replace(/\0/g, '').trim();
}
```

---

### `get_has_file_header(buffer_ptr: i32, buffer_len: i32) → i32`

Returns `1` if the buffer starts with the DLT storage header magic bytes `DLT\x01`
(`44 4C 54 01`), otherwise `0`. Note: this checks only the 4-byte magic pattern;
the full storage header is 16 bytes. When `has_file_header` is true, the standard
header begins at byte 16 (not byte 4).

```js
const hasFile = wasm.get_has_file_header(ptr, len);
```

---

### `get_ecu_id(buffer_ptr: i32, buffer_len: i32) → i32`

Extracts the ECU ID from the standard header as a packed `u32` (little-endian 4
ASCII bytes), or `0` if no ECU ID is present.

```js
const packed = wasm.get_ecu_id(ptr, len);
const ecuId = String.fromCharCode(packed & 0xFF, (packed>>8)&0xFF,
                                  (packed>>16)&0xFF, (packed>>24)&0xFF)
              .replace(/\0/g,'').trim();
```

---

### `get_app_id(buffer_ptr: i32, buffer_len: i32) → i32`

Same as `get_ecu_id` but reads the App ID from the extended header.

---

### `get_context_id(buffer_ptr: i32, buffer_len: i32) → i32`

Same as `get_ecu_id` but reads the Context ID from the extended header.

---

## 7. Message Generation — Log

### `generate_log_message(...) → i32`

Generate a complete DLT log message with custom configuration and payload.

**Signature:**
```
generate_log_message(
  config_ptr: i32,   // pointer to 24-byte config block
  payload_ptr: i32,  // pointer to UTF-8 payload bytes
  payload_len: i32,
  out_ptr: i32,      // pointer to output buffer (min 64 bytes)
  out_len: i32
) → i32              // bytes written, or negative error code
```

**Config block layout (24 bytes):**

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0 | 4 | `ecu_id` | ECU ID (ASCII, null-pad to 4 bytes, e.g. `"ECU1"`) |
| 4 | 4 | `app_id` | Application ID |
| 8 | 4 | `ctx_id` | Context ID |
| 12 | 1 | `log_level` | 1=Fatal, 2=Error, 3=Warn, 4=Info, 5=Debug, 6=Verbose |
| 13 | 1 | `verbose` | 0=non-verbose, 1=verbose payload |
| 14 | 1 | `noar` | Number of payload arguments (usually 1) |
| 15 | 1 | `file_header` | 0=no file header, 1=prepend 16-byte storage header |
| 16 | 4 | `timestamp` | u32 LE, 0.1 ms units (0 = no timestamp) |
| 20 | 4 | reserved | Zero |

**Example (JavaScript):**

```js
function generateLogMessage(wasm, { ecuId, appId, ctxId, logLevel, payload, verbose = false }) {
  const enc = new TextEncoder();
  const payBytes = enc.encode(payload);

  const config = new Uint8Array(24);
  const writeId = (str, offset) => {
    for (let i = 0; i < 4; i++) config[offset + i] = str.charCodeAt(i) || 0;
  };
  writeId(ecuId.padEnd(4, '\0'), 0);
  writeId(appId.padEnd(4, '\0'), 4);
  writeId(ctxId.padEnd(4, '\0'), 8);
  config[12] = logLevel;    // 4 = Info
  config[13] = verbose ? 1 : 0;
  config[14] = 1;           // noar
  config[15] = 0;           // no file header
  // config[16..20] = timestamp (u32 LE, leave 0)

  const cfgPtr = wasm.allocate(24);
  const payPtr = wasm.allocate(payBytes.length);
  const outPtr = wasm.allocate(256);

  new Uint8Array(wasm.memory.buffer, cfgPtr, 24).set(config);
  new Uint8Array(wasm.memory.buffer, payPtr, payBytes.length).set(payBytes);

  const size = wasm.generate_log_message(cfgPtr, payPtr, payBytes.length, outPtr, 256);

  let result = null;
  if (size > 0) {
    result = new Uint8Array(wasm.memory.buffer, outPtr, size).slice();
  }

  wasm.deallocate(cfgPtr);
  wasm.deallocate(payPtr);
  wasm.deallocate(outPtr);
  return { size, bytes: result };
}
```

---

### `create_dlt_message(buffer_ptr: i32, buffer_len: i32) → i32`

Quick helper: generates a hardcoded DLT Info message (`ECU="WASM"`, `APP="TEST"`,
`CTX="DEMO"`, payload=`"Hello from WASM!"`) with serial header.

- `buffer_len` must be ≥ 100.
- Returns bytes written or negative error code.

```js
const ptr = wasm.allocate(256);
const size = wasm.create_dlt_message(ptr, 256);
const bytes = new Uint8Array(wasm.memory.buffer, ptr, size).slice();
wasm.deallocate(ptr);
```

---

### `create_dlt_message_with_file_header(buffer_ptr: i32, buffer_len: i32) → i32`

Same as `create_dlt_message` but prepends the 16-byte DLT storage header.

- `buffer_len` must be ≥ 116.

---

## 8. Message Generation — Service

All service generation functions share a **12-byte config block**:

| Offset | Size | Field |
|--------|------|-------|
| 0 | 4 | `ecu_id` |
| 4 | 4 | `app_id` |
| 8 | 4 | `ctx_id` |

---

### `generate_service_response(config_ptr, service_id, status, out_ptr, out_len) → i32`

Generate a generic service status response.

| Parameter | Type | Description |
|-----------|------|-------------|
| `config_ptr` | i32 | 12-byte config block |
| `service_id` | i32 | Service ID (u32, see table below) |
| `status` | i32 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |
| `out_ptr` | i32 | Output buffer (min 64 bytes) |
| `out_len` | i32 | Output buffer length |

**Service IDs:**

| Name | Value |
|------|-------|
| SetLogLevel | `0x01` |
| SetTraceStatus | `0x02` |
| GetLogInfo | `0x03` |
| GetDefaultLogLevel | `0x04` |
| StoreConfiguration | `0x05` |
| ResetToFactoryDefault | `0x06` |
| SetMessageFiltering | `0x0A` |
| SetDefaultLogLevel | `0x11` |
| GetSoftwareVersion | `0x13` |

---

### `generate_set_log_level_request(config_ptr, target_app_ptr, target_ctx_ptr, log_level, out_ptr, out_len) → i32`

| Parameter | Description |
|-----------|-------------|
| `target_app_ptr` | 4-byte App ID to target |
| `target_ctx_ptr` | 4-byte Context ID to target |
| `log_level` | i8 log level value |

---

### `generate_get_log_info_request(config_ptr, options, target_app_ptr, target_ctx_ptr, out_ptr, out_len) → i32`

| `options` | Meaning |
|-----------|---------|
| `6` | With log level and trace status |
| `7` | With descriptions |

---

### `generate_get_default_log_level_request(config_ptr, out_ptr, out_len) → i32`

Generates a GetDefaultLogLevel control request.

---

### `generate_get_software_version_request(config_ptr, out_ptr, out_len) → i32`

Generates a GetSoftwareVersion control request.

---

### `generate_get_software_version_response(config_ptr, status, version_ptr, version_len, out_ptr, out_len) → i32`

| Parameter | Description |
|-----------|-------------|
| `version_ptr` | Pointer to version string bytes |
| `version_len` | Length of version string |

---

### `generate_get_default_log_level_response(config_ptr, status, log_level, out_ptr, out_len) → i32`

| Parameter | Description |
|-----------|-------------|
| `log_level` | Current default log level byte |

---

## 9. Service Message Parsing

### `parse_service_message(buffer_ptr: i32, buffer_len: i32, result_ptr: i32) → i32`

Parse a complete DLT message and extract service/control information.

- Returns `1` on success, negative error code on failure.
- Writes a 48-byte `ServiceParseResult` to `result_ptr`.

**Output struct layout (48 bytes, LE integers):**

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0 | 4 | `service_id` | u32 LE — parsed service ID (0 for non-control messages) |
| 4 | 1 | `is_response` | 0=request, 1=response |
| 5 | 1 | `status` | Response status byte (0=OK) |
| 6 | 1 | `mstp` | Message type (3=Control) |
| 7 | 1 | `mtin` | 1=request, 2=response |
| 8 | 4 | `ecu_id` | ECU ID bytes |
| 12 | 4 | `app_id` | App ID bytes |
| 16 | 4 | `ctx_id` | Context ID bytes |
| 20 | 2 | `payload_len` | u16 LE — service payload length |
| 22 | 2 | `payload_off` | u16 LE — offset of service payload in input buffer |
| 24 | 4 | `param1` | u32 LE — service-specific (see below) |
| 28 | 4 | `param2` | u32 LE — service-specific (see below) |
| 32 | 1 | `param3` | u8 — service-specific (see below) |
| 33 | 15 | reserved | Zeroed |

**Service-specific parameters:**

| Service | `param1` | `param2` | `param3` |
|---------|----------|----------|----------|
| SetLogLevel (req) | target App ID (4 bytes as u32) | target Ctx ID | log level |
| GetLogInfo (req) | target App ID | target Ctx ID | options |
| SetDefaultLogLevel (req) | — | — | log level |
| SetMessageFiltering (req) | — | — | 0=off, 1=on |
| GetDefaultLogLevel (resp) | — | — | log level |
| GetSoftwareVersion (resp) | version string length | version offset in buffer | — |

```js
function parseServiceMsg(wasm, bytes) {
  const ptr = wasm.allocate(bytes.length);
  const resPtr = wasm.allocate(48);
  new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);

  const rc = wasm.parse_service_message(ptr, bytes.length, resPtr);
  wasm.deallocate(ptr);
  if (rc < 0) { wasm.deallocate(resPtr); return null; }

  const dv = new DataView(wasm.memory.buffer, resPtr, 48);
  const result = {
    serviceId:  dv.getUint32(0, true),
    isResponse: dv.getUint8(4),
    status:     dv.getUint8(5),
    mstp:       dv.getUint8(6),
    mtin:       dv.getUint8(7),
    ecuId:      id4(wasm, resPtr + 8),
    appId:      id4(wasm, resPtr + 12),
    ctxId:      id4(wasm, resPtr + 16),
    payloadLen: dv.getUint16(20, true),
    payloadOff: dv.getUint16(22, true),
    param1:     dv.getUint32(24, true),
    param2:     dv.getUint32(28, true),
    param3:     dv.getUint8(32),
  };
  wasm.deallocate(resPtr);
  return result;
}
```

---

## 10. Payload Formatting

### `format_verbose_payload(buffer_ptr, buffer_len, payload_offset, payload_len, mstp) → i32`

Parse verbose payload arguments and produce a human-readable string.

- Only works for **Log messages** (`mstp == 0`). Returns `ERROR_INVALID_FORMAT` for
  Control/Network messages.
- Supports the template pattern `"{}"` in the first string argument, substituting the
  next integer argument in place of `{}`.
- Returns the byte length of the formatted string, or a negative error code.
- The result string is stored internally; retrieve it with `get_formatted_payload_ptr()`.

**Parameters:**

| Parameter | Source |
|-----------|--------|
| `buffer_ptr` | Pointer to full DLT message in WASM memory |
| `buffer_len` | Total length of DLT message |
| `payload_offset` | Value from `AnalysisResult.payload_offset` (offset 6) |
| `payload_len` | Value from `AnalysisResult.payload_len` (offset 4) |
| `mstp` | Value from `AnalysisResult.mstp` (offset 24) — must be 0 |

### `get_formatted_payload_ptr() → i32`

Returns a pointer to the internally stored formatted string (valid until the next call
to `format_verbose_payload`). Returns `0` if nothing has been formatted.

```js
function formatPayload(wasm, msgBytes, analysis) {
  const ptr = wasm.allocate(msgBytes.length);
  new Uint8Array(wasm.memory.buffer, ptr, msgBytes.length).set(msgBytes);

  const len = wasm.format_verbose_payload(
    ptr,
    msgBytes.length,
    analysis.payloadOffset,
    analysis.payloadLen,
    analysis.mstp
  );
  wasm.deallocate(ptr);

  if (len <= 0) return null;

  const strPtr = wasm.get_formatted_payload_ptr();
  if (!strPtr) return null;

  return new TextDecoder().decode(
    new Uint8Array(wasm.memory.buffer, strPtr, len)
  );
}
```

---

## 11. Utilities

### `get_version() → i32`

Returns the API version as an integer. Current value: `100` (= v1.0.0).

```js
const ver = wasm.get_version(); // 100
console.log(`v${Math.floor(ver/100)}.${Math.floor((ver%100)/10)}.${ver%10}`); // "v1.0.0"
```

---

## 12. DLT Message Structure Reference

### Standard wire format (no file header):

```
[Serial Header 4 bytes] optional  "DLS\x01" = 44 4C 53 01
[Standard Header 4 bytes]         HTYP | MCNT | LEN(u16BE)
[ECU ID 4 bytes]          if WEID bit set
[Session ID 4 bytes]      if WSID bit set
[Timestamp 4 bytes]       if WTMS bit set (units: 0.1 ms)
[Extended Header 10 bytes] if UEH bit set
  [MSIN 1 byte]
  [NOAR 1 byte]
  [APID 4 bytes]
  [CTID 4 bytes]
[Payload N bytes]
```

### With storage header (for .dlt file storage):

```
[Storage Header  16 bytes]
  [Pattern   4 bytes]   "DLT\x01" = 44 4C 54 01
  [Seconds   4 bytes]   u32 little-endian, Unix epoch seconds
  [Microsecs 4 bytes]   u32 little-endian, microseconds part
  [ECU ID    4 bytes]   ASCII, null-padded (logger's ECU ID)
[Serial Header    4 bytes]   optional "DLS\x01"
[Standard Header …]
```

### HTYP bitmask:

| Bit | Mask | Name | Meaning |
|-----|------|------|---------|
| 0 | `0x01` | UEH | Use Extended Header |
| 1 | `0x02` | MSBF | Big-endian payload |
| 2 | `0x04` | WEID | ECU ID present |
| 3 | `0x08` | WSID | Session ID present |
| 4 | `0x10` | WTMS | Timestamp present |
| 5–7 | `0xE0` | VERS | Protocol version (should be 1) |

### MSIN byte (Extended Header byte 0):

| Bits | Field | Meaning |
|------|-------|---------|
| 0 | Verbose | 1 = verbose payload |
| 1–3 | MSTP | Message Type (0=Log, 1=Trace, 2=Network, 3=Control) |
| 4–7 | MTIN | Message Type Info (log level for MSTP=0) |

### Log level MTIN values (MSTP = 0):

| MTIN | Value | Name |
|------|-------|------|
| 0 | — | (none/default) |
| 1 | Fatal | Fatal |
| 2 | Error | Error |
| 3 | Warn | Warning |
| 4 | Info | Info |
| 5 | Debug | Debug |
| 6 | Verbose | Verbose |

### Verbose payload argument frame:

```
[Type Info  4 bytes LE]  see type flags below
[Length     2 bytes LE]  for string/raw types only
[Data       N bytes]
```

**Type Info flags (bit positions):**

| Bits | Meaning |
|------|---------|
| 0–3 | Type length: 1=8b, 2=16b, 3=32b, 4=64b, 5=128b |
| 4 | BOOL |
| 8 | UINT |
| 12 | SINT |
| 9 | `0x0200` STRG (string) |
| 10 | `0x0400` RAWD (raw data) |

---

## 13. Constants Reference

These byte arrays identify DLT frame boundaries:

| Constant | Value | Use |
|----------|-------|-----|
| `DLT_SERIAL_HEADER_ARRAY` | `44 4C 53 01` ("DLS\x01") | Serial framing header |
| `DLT_FILE_HEADER_ARRAY` | `44 4C 54 01` ("DLT\x01") | Storage header magic (first 4 of 16 bytes) |
| `DLT_STORAGE_HEADER_SIZE` | `16` | Full storage header byte count |
| `DLT_FILE_HEADER_SIZE` | `4` | Magic pattern byte count (detection only) |

Minimum buffer sizes:

| Scenario | Minimum bytes |
|----------|---------------|
| `create_dlt_message` | 100 |
| `create_dlt_message_with_file_header` | 116 |
| `generate_log_message` output | 64 |
| `generate_service_*` output | 64 |
| `analyze_dlt_message` result | 32 (returned from allocator) |
| `parse_service_message` result | 48 (caller allocated) |

---

## 14. Complete Node.js Example

```js
#!/usr/bin/env node
// Usage: node client.mjs
import { readFileSync } from 'fs';
import { createConnection } from 'net';

const wasm = (await WebAssembly.instantiate(
  readFileSync('target/wasm32-unknown-unknown/release/examples/wasm_demo.wasm'), {}
)).instance.exports;

const LOG_LEVELS = ['?', 'FATAL', 'ERROR', 'WARN', 'INFO', 'DEBUG', 'VERBOSE'];

function id4(offset) {
  return String.fromCharCode(
    ...new Uint8Array(wasm.memory.buffer, offset, 4)
  ).replace(/\0/g, '').trim() || '----';
}

function parseMessage(data, offset) {
  if (data.length - offset < 4) return null;

  // Detect optional file/serial headers to find standard header
  let std = offset;
  // Storage header is 16 bytes: pattern(4) + seconds(4) + microseconds(4) + ECU ID(4)
  if (data[std]===0x44&&data[std+1]===0x4C&&data[std+2]===0x54&&data[std+3]===0x01) std+=16;
  if (data.length-std>=4 &&
      data[std]===0x44&&data[std+1]===0x4C&&data[std+2]===0x53&&data[std+3]===0x01) std+=4;
  if (data.length - std < 4) return null;

  const msgLen = (data[std + 2] << 8) | data[std + 3]; // big-endian LEN field
  const total = (std - offset) + msgLen;
  if (data.length - offset < total) return null;

  const ptr = wasm.allocate(total);
  if (!ptr) return null;
  new Uint8Array(wasm.memory.buffer, ptr, total).set(data.subarray(offset, offset + total));

  const rPtr = wasm.analyze_dlt_message(ptr, total);
  wasm.deallocate(ptr);
  if (!rPtr) return { parsed: null, consumed: total };

  const dv = new DataView(wasm.memory.buffer, rPtr, 32);
  const parsed = {
    ecuId:         id4(rPtr + 12),
    appId:         id4(rPtr + 16),
    ctxId:         id4(rPtr + 20),
    logLevel:      dv.getUint8(9),
    mstp:          dv.getUint8(24),
    isVerbose:     dv.getUint8(25),
    payloadOffset: dv.getUint16(6, true),
    payloadLen:    dv.getUint16(4, true),
  };
  wasm.deallocate(rPtr);

  // Decode payload text
  let payloadText = '';
  if (parsed.payloadLen > 0) {
    const pBytes = data.subarray(offset + parsed.payloadOffset,
                                  offset + parsed.payloadOffset + parsed.payloadLen);
    payloadText = Buffer.from(parsed.isVerbose && parsed.payloadLen > 4
      ? pBytes.subarray(4) : pBytes).toString('utf8').replace(/\0/g, '');
  }

  return { parsed: { ...parsed, payloadText }, consumed: total };
}

let rxBuf = Buffer.alloc(0);
let count = 0;

const sock = createConnection({ host: 'localhost', port: 3490 }, () =>
  console.log('Connected to DLT TCP server\n')
);

sock.on('data', chunk => {
  rxBuf = Buffer.concat([rxBuf, chunk]);
  let off = 0;
  while (off < rxBuf.length) {
    const r = parseMessage(rxBuf, off);
    if (!r) break;
    if (r.parsed) {
      const { ecuId, appId, ctxId, logLevel, payloadText } = r.parsed;
      console.log(`[${++count}] ${ecuId} ${appId}/${ctxId} [${LOG_LEVELS[logLevel]??'?'}] "${payloadText}"`);
    }
    off += r.consumed;
  }
  rxBuf = rxBuf.subarray(off);
});

sock.on('end', () => console.log(`\nDone. ${count} messages.`));
process.on('SIGINT', () => { sock.destroy(); process.exit(0); });
```

---

## 15. Complete Browser Example

```html
<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"><title>DLT WASM Parser</title></head>
<body>
<pre id="out"></pre>
<script type="module">
const { instance } = await WebAssembly.instantiateStreaming(fetch('wasm_demo.wasm'), {});
const wasm = instance.exports;

// Connect via WebSocket bridge (see dlt_websocket_bridge.py)
const ws = new WebSocket('ws://localhost:8765');
ws.binaryType = 'arraybuffer';
let rxBuf = new Uint8Array(0);

const LOG_LEVELS = ['?','FATAL','ERROR','WARN','INFO','DEBUG','VERBOSE'];

function id4(off) {
  return String.fromCharCode(...new Uint8Array(wasm.memory.buffer, off, 4))
    .replace(/\0/g,'').trim() || '----';
}

ws.onopen = () => {
  ws.send(JSON.stringify({ cmd: 'connect', host: 'localhost', port: 3490 }));
};

ws.onmessage = ({ data }) => {
  if (typeof data === 'string') {
    // JSON status from bridge
    const msg = JSON.parse(data);
    console.log('Bridge:', msg.status);
    return;
  }

  // Append binary DLT data
  const chunk = new Uint8Array(data);
  const merged = new Uint8Array(rxBuf.length + chunk.length);
  merged.set(rxBuf); merged.set(chunk, rxBuf.length);
  rxBuf = merged;

  let offset = 0;
  while (offset < rxBuf.length) {
    let std = offset;
    // Storage header is 16 bytes: pattern(4) + seconds(4) + microseconds(4) + ECU ID(4)
    if (rxBuf[std]===0x44&&rxBuf[std+1]===0x4C&&rxBuf[std+2]===0x54&&rxBuf[std+3]===0x01) std+=16;
    if (rxBuf.length-std>=4&&rxBuf[std]===0x44&&rxBuf[std+1]===0x4C&&rxBuf[std+2]===0x53&&rxBuf[std+3]===0x01) std+=4;
    if (rxBuf.length - std < 4) break;

    const total = (std - offset) + ((rxBuf[std+2]<<8)|rxBuf[std+3]);
    if (rxBuf.length - offset < total) break;

    const ptr = wasm.allocate(total);
    new Uint8Array(wasm.memory.buffer, ptr, total).set(rxBuf.subarray(offset, offset+total));
    const rPtr = wasm.analyze_dlt_message(ptr, total);
    wasm.deallocate(ptr);

    if (rPtr) {
      const dv = new DataView(wasm.memory.buffer, rPtr, 32);
      const lvl = dv.getUint8(9);
      const pOff = dv.getUint16(6, true), pLen = dv.getUint16(4, true);
      const pBytes = rxBuf.subarray(offset+pOff, offset+pOff+pLen);
      const text = new TextDecoder().decode(dv.getUint8(25) && pLen>4 ? pBytes.subarray(4) : pBytes)
                     .replace(/\0/g,'');
      document.getElementById('out').textContent +=
        `${id4(rPtr+12)} ${id4(rPtr+16)}/${id4(rPtr+20)} [${LOG_LEVELS[lvl]??'?'}] "${text}"\n`;
      wasm.deallocate(rPtr);
    }
    offset += total;
  }
  rxBuf = rxBuf.subarray(offset);
};
</script>
</body>
</html>
```
