# DLT Daemon Simple Example

A minimal DLT (Diagnostic Log and Trace) daemon implementation that demonstrates:
- TCP server listening on `localhost:3490`
- Service message request/response handling
- Periodic log message broadcasting
- Multi-client support

##Usage

### Start the Daemon

```bash
cargo run --example dlt_daemon_simple
```

The daemon will start and listen on `localhost:3490`.

### Connect a Client

In another terminal, connect with the console viewer:

```bash
cargo run --example dlt_console_viewer
```

## Features

### Service Messages Supported

The daemon responds to the following DLT service messages:

- ✅ **SetLogLevel** (0x01) - Set log level for app/context
- ✅ **SetDefaultLogLevel** (0x11) - Set global default log level
- ✅ **GetDefaultLogLevel** (0x04) - Query current default log level
- ✅ **GetSoftwareVersion** (0x13) - Get daemon version info
- ✅ **SetMessageFiltering** (0x0A) - Enable/disable message filtering
- ✅ **StoreConfiguration** (0x05) - Save current configuration
- ✅ **ResetToFactoryDefault** (0x06) - Reset to default settings
- ⚠️  Other services return `NotSupported`

### Log Messages

The daemon automatically sends:
- **Welcome message** when a client connects
- **Periodic heartbeat** every 5 seconds
- **Event notifications** for configuration changes

## Message Flow

```
Client                          Daemon
  |                               |
  |-------- Connect TCP -------->|
  |                               |
  |<----- Welcome Log Msg -------|
  |                               |
  |---- Service Request (0x01)-->|
  |                               |
  |<---- Service Response -------|
  |                               |
  |<--- Periodic Heartbeat ------|
  |     (every 5 seconds)         |
```

## Testing Service Requests

You can send service requests by creating a simple client. Example:

```rust
use dlt_protocol::r19_11::*;
use std::net::TcpStream;
use std::io::Write;

let mut stream = TcpStream::connect("localhost:3490")?;
let mut buffer = [0u8; 256];

// Create SetLogLevel request
let mut builder = DltServiceMessageBuilder::new()
    .with_ecu_id(b"TEST")
    .with_app_id(b"APP1")
    .with_context_id(b"CTX1")
    .add_serial_header();

let len = builder.generate_set_log_level_request(
    &mut buffer,
    b"APP2",
    b"CTX2",
    MtinTypeDltLog::DltLogDebug.to_bits() as i8,
)?;

stream.write_all(&buffer[..len])?;
```

## Implementation Details

### State Management
- **Thread-safe** using `Arc<Mutex<DaemonState>>`
- Each client connection runs in separate thread
- Shared state for configuration (log levels, settings)

### Protocol Compliance
- Full DLT R19-11 protocol support
- Serial header enabled for all messages
- Big-endian service ID encoding
- Proper message length calculation

### Error Handling
- Graceful handling of malformed messages
- Client disconnection detection
- Parse error logging to console

## Example Output

```
🚀 DLT Daemon - Simple Example
Listening on localhost:3490...

✅ DLT Daemon started successfully!
================================================================================
📡 New connection from: 127.0.0.1:54321
🔧 Service Request: SetLogLevel from APP1:CTX1
  → SetLogLevel: "APP2":"CTX2" = 5
🔧 Service Request: GetSoftwareVersion from APP1:CTX1
  → GetSoftwareVersion: 1.0.0
📝 Log message received: level=DltLogDebug
```

## Architecture

```
┌─────────────────────────────────────┐
│       DLT Daemon (Port 3490)        │
│                                     │
│  ┌───────────────────────────────┐ │
│  │    TCP Listener Thread        │ │
│  │  - Accept connections         │ │
│  │  - Spawn client handlers      │ │
│  └───────────────────────────────┘ │
│                                     │
│  ┌───────────────────────────────┐ │
│  │  Client Handler Thread 1      │ │
│  │  - Parse messages             │ │
│  │  - Process service requests   │ │
│  │  - Send responses             │ │
│  │  - Periodic log sender        │ │
│  └───────────────────────────────┘ │
│                                     │
│  ┌───────────────────────────────┐ │
│  │  Client Handler Thread 2...   │ │
│  └───────────────────────────────┘ │
│                                     │
│  ┌───────────────────────────────┐ │
│  │   Shared Daemon State         │ │
│  │   (Arc<Mutex<...>>)           │ │
│  │  - Default log level          │ │
│  │  - Message filtering          │ │
│  │  - Software version           │ │
│  └───────────────────────────────┘ │
└─────────────────────────────────────┘
```

## Limitations

This is a **simple example** for demonstration purposes:

- No persistence (state lost on restart)
- Basic error recovery
- Limited service message implementations
- No authentication or security
- Single-threaded message processing per client

For production use, consider:
- Persistent storage for configuration
- More robust error handling
- Complete service message support
- Security/authentication
- Performance optimization
