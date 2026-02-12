# DLT Stream Viewer - Real-time Message Parser

This setup allows you to visualize DLT messages from dlt-daemon in your browser using WebAssembly.

## Architecture

```
dlt-daemon (TCP :3490) → WebSocket Bridge → Browser (WASM Parser) → Visual Display
```

## Quick Start

### 1. Start dlt-daemon

Make sure dlt-daemon is running and listening on TCP port 3490:

```bash
# Check if dlt-daemon is running
ps aux | grep dlt-daemon

# If not running, start it (example):
dlt-daemon -d
```

### 2. Install WebSocket Library (if needed)

```bash
pip3 install websockets
```

### 3. Build WASM Module

```bash
cd /Users/sentama/Development/tama/rust/dlt-protocol
./build-wasm.sh
```

### 4. Start HTTP Server (in one terminal)

```bash
python3 -m http.server 8000
```

### 5. Start WebSocket Bridge (in another terminal)

```bash
python3 examples/dlt_websocket_bridge.py
```

You should see:
```
🚀 DLT WebSocket Bridge Server
============================================================
📡 DLT daemon:    localhost:3490
🌐 WebSocket:     ws://localhost:8765
📄 Open browser:  http://localhost:8000/examples/dlt_stream.html
============================================================
✅ WebSocket server running on port 8765
Press Ctrl+C to stop
```

### 6. Open Browser

Navigate to: http://localhost:8000/examples/dlt_stream.html

Click "Connect" to start receiving and parsing DLT messages in real-time!

## Features

- ✅ Real-time DLT message parsing using WASM
- ✅ Auto-scrolling message display
- ✅ Color-coded log levels (Fatal, Error, Warning, Info, Debug, Verbose)
- ✅ Live statistics (messages/sec, bytes received, parse errors)
- ✅ Verbose payload formatting (template string support)
- ✅ ECU/App/Context ID extraction
- ✅ Message filtering and display

## Configuration

### Custom WebSocket URL

You can connect to different WebSocket servers by changing the URL in the browser interface.

### Custom DLT Daemon Port

```bash
python3 examples/dlt_websocket_bridge.py --dlt-port 3491 --ws-port 8765
```

### Command-line Options

```bash
python3 examples/dlt_websocket_bridge.py --help

Options:
  --dlt-host HOST   DLT daemon hostname (default: localhost)
  --dlt-port PORT   DLT daemon port (default: 3490)
  --ws-port PORT    WebSocket server port (default: 8765)
```

## Troubleshooting

### "Connection refused to localhost:3490"

- Make sure dlt-daemon is running
- Check that it's listening on port 3490: `netstat -an | grep 3490`

### "Failed to load WASM"

- Make sure you ran `./build-wasm.sh`
- Check that HTTP server is running on port 8000
- Verify file exists: `ls target/wasm32-unknown-unknown/release/examples/wasm_demo.wasm`

### "WebSocket connection failed"

- Make sure the bridge script is running
- Check the WebSocket URL in the browser (default: ws://localhost:8765)
- Check firewall settings

### No messages appearing

- Verify dlt-daemon is receiving messages
- Check browser console for errors (F12)
- Try sending test messages to dlt-daemon

## Testing with dlt-daemon

Send test messages using dlt-logstorage-ctrl or any DLT client:

```bash
# Example: Send test log
echo "Test message" | dlt-adaptor-stdin TEST CTX1
```

## Performance

The WASM parser can handle hundreds of messages per second. The "Messages/sec" counter in the UI shows real-time throughput.

## Payload Parsing

- **Non-verbose mode**: Raw binary payload displayed as text
- **Verbose mode**: Type-aware parsing with template string support
  - Example: "Temperature: {} °C" with integer argument → "Temperature: 25 °C"

## Browser Compatibility

Tested on:
- ✅ Chrome/Edge (Chromium-based)
- ✅ Firefox
- ✅ Safari (macOS)

Requires WebAssembly and WebSocket support (all modern browsers).
