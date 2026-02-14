#!/usr/bin/env python3
"""
Simple DLT Packet Hex Viewer

Connects to a DLT daemon on localhost:3490 and prints each received
DLT packet in hexadecimal format.

Usage:
    python3 dlt_hex_viewer.py

Prerequisites:
    - DLT daemon running on localhost:3490
    - Start with: cargo run --example dlt_daemon_simple
"""

import socket
import struct
import sys


def format_hex(data, bytes_per_line=16):
    """Format bytes as hex dump with ASCII representation"""
    lines = []
    for i in range(0, len(data), bytes_per_line):
        chunk = data[i:i + bytes_per_line]
        
        # Hex part
        hex_part = ' '.join(f'{b:02x}' for b in chunk)
        hex_part = hex_part.ljust(bytes_per_line * 3 - 1)
        
        # ASCII part (printable chars only)
        ascii_part = ''.join(chr(b) if 32 <= b < 127 else '.' for b in chunk)
        
        lines.append(f'  {i:04x}  {hex_part}  {ascii_part}')
    
    return '\n'.join(lines)


def read_dlt_packet(sock):
    """Read one DLT packet from socket"""
    # Read standard header (4 bytes)
    std_header = sock.recv(4)
    if len(std_header) < 4:
        return None
    
    # Extract message length from bytes 2-3 (big-endian)
    msg_len = struct.unpack('>H', std_header[2:4])[0]
    
    if msg_len < 4 or msg_len > 65535:
        print(f"⚠️  Invalid message length: {msg_len}")
        return None
    
    # Read remaining bytes
    remaining = msg_len - 4
    message = bytearray(std_header)
    
    while remaining > 0:
        chunk = sock.recv(remaining)
        if not chunk:
            return None
        message.extend(chunk)
        remaining -= len(chunk)
    
    return bytes(message)


def main():
    print("🔍 DLT Packet Hex Viewer")
    print("=" * 80)
    print("Connecting to localhost:3490...\n")
    
    try:
        # Connect to DLT daemon
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect(('localhost', 3490))
        sock.settimeout(5.0)
        
        print("✅ Connected to DLT daemon!")
        print("=" * 80)
        print()
        
        packet_count = 0
        
        while True:
            try:
                # Read one DLT packet
                packet = read_dlt_packet(sock)
                
                if packet is None:
                    print("\n📡 Connection closed by daemon")
                    break
                
                packet_count += 1
                
                # Print packet info
                print(f"📦 Packet #{packet_count} ({len(packet)} bytes)")
                print("-" * 80)
                print(format_hex(packet))
                print()
                
            except socket.timeout:
                continue
            except KeyboardInterrupt:
                print("\n\n⏹️  Interrupted by user")
                break
            except Exception as e:
                print(f"\n❌ Error reading packet: {e}")
                break
        
        sock.close()
        print(f"\n✅ Total packets received: {packet_count}")
        
    except ConnectionRefusedError:
        print("❌ Connection refused!")
        print("   Make sure the DLT daemon is running:")
        print("   cargo run --example dlt_daemon_simple")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Error: {e}")
        sys.exit(1)


if __name__ == '__main__':
    main()
