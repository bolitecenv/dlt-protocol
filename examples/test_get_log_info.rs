/// Test client for GetLogInfo service request
///
/// This example demonstrates sending a GetLogInfo request to the DLT daemon
/// and parsing the response.
///
/// Usage:
/// 1. Start the daemon: cargo run --example dlt_daemon_simple
/// 2. Run this test: cargo run --example test_get_log_info

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use dlt_protocol::r19_11::*;

fn main() -> std::io::Result<()> {
    println!("🧪 Testing GetLogInfo Request");
    println!("Connecting to daemon at localhost:3490...\n");

    let mut stream = TcpStream::connect("localhost:3490")?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    
    println!("✅ Connected!");

    // Read and display initial messages from daemon
    let mut buffer = [0u8; 4096];
    read_initial_messages(&mut stream, &mut buffer)?;

    println!("\n📤 Sending GetLogInfo request (option 6 - with log level and trace status)...");
    send_get_log_info_request(&mut stream, 6)?;

    println!("📥 Waiting for response...\n");
    std::thread::sleep(Duration::from_millis(500));

    // Read GetLogInfo response
    read_get_log_info_response(&mut stream, &mut buffer)?;

    println!("\n📤 Sending GetLogInfo request (option 7 - with descriptions)...");
    send_get_log_info_request(&mut stream, 7)?;

    println!("📥 Waiting for response...\n");
    std::thread::sleep(Duration::from_millis(500));

    // Read GetLogInfo response with descriptions
    read_get_log_info_response(&mut stream, &mut buffer)?;

    println!("\n✅ Test completed successfully!");
    Ok(())
}

fn read_initial_messages(stream: &mut TcpStream, buffer: &mut [u8; 4096]) -> std::io::Result<()> {
    println!("📨 Reading initial messages from daemon...");
    
    for i in 0..3 {
        match read_one_message(stream, buffer) {
            Ok(Some(_)) => {
                println!("  ✓ Message {} received", i + 1);
            }
            Ok(None) => break,
            Err(e) => {
                if i == 0 {
                    return Err(e);
                }
                break;
            }
        }
    }
    
    Ok(())
}

fn read_one_message(stream: &mut TcpStream, buffer: &mut [u8; 4096]) -> std::io::Result<Option<usize>> {
    let mut std_header = [0u8; 4];
    match stream.read_exact(&mut std_header) {
        Ok(_) => {},
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || 
                  e.kind() == std::io::ErrorKind::TimedOut => {
            return Ok(None);
        }
        Err(e) => return Err(e),
    }

    let msg_len = u16::from_be_bytes([std_header[2], std_header[3]]) as usize;
    
    if msg_len < 4 || msg_len > buffer.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid message length: {}", msg_len),
        ));
    }

    buffer[0..4].copy_from_slice(&std_header);
    
    let remaining = msg_len - 4;
    if remaining > 0 {
        stream.read_exact(&mut buffer[4..4 + remaining])?;
    }

    Ok(Some(msg_len))
}

fn send_get_log_info_request(stream: &mut TcpStream, options: u8) -> std::io::Result<()> {
    let mut buffer = [0u8; 256];
    let mut builder = DltServiceMessageBuilder::new()
        .with_ecu_id(b"TEST")
        .with_app_id(b"TEST")
        .with_context_id(b"TST1");

    // Request all apps and contexts (wildcard)
    let len = builder.generate_get_log_info_request(
        &mut buffer,
        options,
        &[0, 0, 0, 0], // wildcard app_id
        &[0, 0, 0, 0], // wildcard ctx_id
    ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", e)))?;

    stream.write_all(&buffer[..len])?;
    stream.flush()?;
    
    Ok(())
}

fn read_get_log_info_response(stream: &mut TcpStream, buffer: &mut [u8; 4096]) -> std::io::Result<()> {
    match read_one_message(stream, buffer) {
        Ok(Some(msg_len)) => {
            let message = &buffer[..msg_len];
            
            // Parse the message
            let mut parser = DltHeaderParser::new(message);
            match parser.parse_message() {
                Ok(msg) => {
                    if let Some(ext_hdr) = msg.extended_header {
                        if matches!(ext_hdr.message_type(), MstpType::DltTypeControl) {
                            let service_parser = DltServiceParser::new(msg.payload);
                            
                            match service_parser.parse_service_id() {
                                Ok(ServiceId::GetLogInfo) => {
                                    // Debug: print payload info
                                    let payload = service_parser.get_payload();
                                    println!("   DEBUG: Payload size = {} bytes", payload.len());
                                    if payload.len() >= 4 {
                                        println!("   DEBUG: Last 4 bytes: {:02x} {:02x} {:02x} {:02x}", 
                                            payload[payload.len()-4], payload[payload.len()-3], 
                                            payload[payload.len()-2], payload[payload.len()-1]);
                                    }
                                    if payload.len() < 20 {
                                        print!("   DEBUG: Full payload hex: ");
                                        for b in payload {
                                            print!("{:02x} ", b);
                                        }
                                        println!();
                                    }
                                    
                                    match service_parser.parse_get_log_info_response() {
                                        Ok((status, log_info_data)) => {
                                            println!("📋 GetLogInfo Response - Status: {:?}", status);
                                            
                                            let with_desc = matches!(status, ServiceStatus::WithDescriptions);
                                            let mut info_parser = LogInfoResponseParser::new(log_info_data, with_desc);
                                            
                                            match info_parser.read_app_count() {
                                                Ok(app_count) => {
                                                    println!("   Applications: {}", app_count);
                                                    
                                                    for app_idx in 0..app_count {
                                                        if let Ok(app_id) = info_parser.read_app_id() {
                                                            let app_str = String::from_utf8_lossy(&app_id).trim_end_matches('\0').to_string();
                                                            
                                                            if let Ok(ctx_count) = info_parser.read_context_count() {
                                                                println!("\n   App[{}]: \"{}\" ({} contexts)", app_idx, app_str, ctx_count);
                                                                
                                                                for ctx_idx in 0..ctx_count {
                                                                    if let Ok((ctx_id, log_lvl, trace)) = info_parser.read_context_info() {
                                                                        let ctx_str = String::from_utf8_lossy(&ctx_id).trim_end_matches('\0').to_string();
                                                                        print!("     Context[{}]: \"{}\" [LogLevel={}, Trace={}]", 
                                                                            ctx_idx, ctx_str, log_lvl, trace);
                                                                        
                                                                        if with_desc {
                                                                            if let Ok(desc) = info_parser.read_description() {
                                                                                if !desc.is_empty() {
                                                                                    print!(" - \"{}\"", String::from_utf8_lossy(desc));
                                                                                }
                                                                            }
                                                                        }
                                                                        println!();
                                                                    }
                                                                }
                                                                
                                                                if with_desc {
                                                                    if let Ok(app_desc) = info_parser.read_description() {
                                                                        if !app_desc.is_empty() {
                                                                            println!("     AppDescription: \"{}\"", String::from_utf8_lossy(app_desc));
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                },
                                                Err(e) => {
                                                    println!("   ⚠️  Parse error: {:?}", e);
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            println!("   ⚠️  Failed to parse GetLogInfo response: {:?}", e);
                                        }
                                    }
                                },
                                Ok(other) => {
                                    println!("   Received different service: {:?}", other);
                                },
                                Err(e) => {
                                    println!("   ⚠️  Failed to parse service ID: {:?}", e);
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    println!("   ⚠️  Failed to parse message: {:?}", e);
                }
            }
        },
        Ok(None) => {
            println!("   ⚠️  No response received");
        },
        Err(e) => {
            println!("   ⚠️  Error reading response: {}", e);
        }
    }
    
    Ok(())
}
