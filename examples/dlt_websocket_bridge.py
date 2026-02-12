#!/usr/bin/env python3
"""
DLT WebSocket Bridge Server - Raw Passthrough

Connects to dlt-daemon TCP server (port 3490) and forwards raw data
to WebSocket clients for browser-based analysis.
"""

import asyncio
import websockets
import argparse
import sys

DLT_HOST = 'localhost'
DLT_PORT = 3490
WS_PORT = 8765

websocket_clients = set()
message_queue = None

async def read_from_dlt():
    while True:
        try:
            print(f"🔌 Connecting to dlt-daemon at {DLT_HOST}:{DLT_PORT}...")
            reader, writer = await asyncio.open_connection(DLT_HOST, DLT_PORT)
            print("✅ Connected to dlt-daemon!")
            
            while True:
                data = await reader.read(4096)
                if not data:
                    print("⚠️  dlt-daemon connection closed")
                    break
                await message_queue.put(data)
                print(f"📦 Received {len(data)} bytes, hex: {data[:32].hex()}")
                
        except asyncio.CancelledError:
            break
        except ConnectionRefusedError:
            print(f"❌ Connection refused to {DLT_HOST}:{DLT_PORT}")
            await asyncio.sleep(5)
        except Exception as e:
            print(f"❌ Error: {e}")
            await asyncio.sleep(5)
        finally:
            if 'writer' in locals():
                writer.close()
                await writer.wait_closed()

async def broadcast_messages():
    while True:
        try:
            data = await message_queue.get()
            if websocket_clients:
                print(f"📤 Broadcasting {len(data)} bytes to {len(websocket_clients)} client(s)")
                for ws in list(websocket_clients):
                    try:
                        await ws.send(data)
                    except Exception as e:
                        print(f"⚠️  Error sending: {e}")
                        websocket_clients.discard(ws)
            else:
                print(f"⚠️  No clients, data dropped")
        except asyncio.CancelledError:
            break

async def websocket_handler(websocket):
    print(f"🌐 Client connected: {websocket.remote_address}")
    websocket_clients.add(websocket)
    try:
        await websocket.wait_closed()
    finally:
        websocket_clients.discard(websocket)
        print(f"👋 Client disconnected")

async def main():
    global DLT_HOST, DLT_PORT, WS_PORT, message_queue
    parser = argparse.ArgumentParser()
    parser.add_argument('--dlt-host', default=DLT_HOST)
    parser.add_argument('--dlt-port', type=int, default=DLT_PORT)
    parser.add_argument('--ws-port', type=int, default=WS_PORT)
    args = parser.parse_args()
    DLT_HOST, DLT_PORT, WS_PORT = args.dlt_host, args.dlt_port, args.ws_port
    
    # Create queue inside async context
    message_queue = asyncio.Queue()
    
    print("🚀 DLT WebSocket Bridge (Raw Passthrough)")
    print(f"📡 DLT: {DLT_HOST}:{DLT_PORT}")
    print(f"🌐 WebSocket: ws://localhost:{WS_PORT}")
    
    dlt_task = asyncio.create_task(read_from_dlt())
    broadcast_task = asyncio.create_task(broadcast_messages())
    
    async with websockets.serve(websocket_handler, 'localhost', WS_PORT):
        print(f"✅ Running on port {WS_PORT}")
        try:
            await asyncio.gather(dlt_task, broadcast_task)
        except KeyboardInterrupt:
            dlt_task.cancel()
            broadcast_task.cancel()

if __name__ == '__main__':
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n👋 Shutting down...")
