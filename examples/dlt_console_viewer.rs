use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use dlt_protocol::r19_11::*;

fn main() -> std::io::Result<()> {
    println!("🚀 DLT Console Viewer");
    println!("Connecting to dlt-daemon at localhost:3490...\n");

    let mut stream = TcpStream::connect("localhost:3490")?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    
    println!("✅ Connected to dlt-daemon!");
    println!("{}", "=".repeat(80));
    
    let mut message_count = 0u32;
    let mut buffer = [0u8; 4096];

    loop {       
        // Read standard header (4 bytes)
        let mut std_header = [0u8; 4];
        match stream.read_exact(&mut std_header) {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || 
                      e.kind() == std::io::ErrorKind::TimedOut => {
                // No data available, wait and try again
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                eprintln!("\n❌ Error reading standard header: {}", e);
                break;
            }
        }

        // Extract message length (bytes 2-3 of standard header, big-endian)
        let msg_len = u16::from_be_bytes([std_header[2], std_header[3]]) as usize;
        
        if msg_len < 4 || msg_len > buffer.len() {
            eprintln!("⚠️  Invalid message length: {}", msg_len);
            continue;
        }

        // Copy standard header to buffer
        buffer[0..4].copy_from_slice(&std_header);

        // Read rest of message
        let remaining = msg_len - 4;
        if remaining > 0 {
            match stream.read_exact(&mut buffer[4..4 + remaining]) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("❌ Error reading message body: {}", e);
                    continue;
                }
            }
        }

        // Analyze the message
        message_count += 1;
        let message = &buffer[..msg_len];
        
        analyze_and_display(message, message_count);
    }

    Ok(())
}

fn analyze_and_display(message: &[u8], msg_num: u32) {
    let htyp = message[0];
    let has_ecu = (htyp & WEID_MASK) != 0;
    let has_session = (htyp & WSID_MASK) != 0;
    let has_timestamp = (htyp & WTMS_MASK) != 0;
    let has_extended = (htyp & UEH_MASK) != 0;
    
    let total_len = u16::from_be_bytes([message[2], message[3]]);
    
    // Calculate header size
    let mut header_len = 4usize;
    if has_ecu { header_len += 4; }
    if has_session { header_len += 4; }
    if has_timestamp { header_len += 4; }
    if has_extended { header_len += 10; }
    
    let payload_len = if total_len as usize > header_len {
        total_len as usize - header_len
    } else {
        0
    };

    // Extract ECU ID
    let mut ecu_id = String::new();
    if has_ecu && message.len() >= 8 {
        ecu_id = bytes_to_string(&message[4..8]);
    }

    // Calculate offset to extended header
    let mut ext_offset = 4;
    if has_ecu { ext_offset += 4; }
    if has_session { ext_offset += 4; }
    if has_timestamp { ext_offset += 4; }

    // Extract message type, log level, App ID, Context ID
    let mut message_type = String::from("Unknown");
    let mut log_level_name = String::from("");
    let mut app_id = String::new();
    let mut ctx_id = String::new();
    
    if has_extended && message.len() >= ext_offset + 10 {
        let msin = message[ext_offset];
        
        // Extract MSTP (Message Type) - bits 1-3
        let mstp = (msin >> 1) & 0x07;
        
        match mstp {
            0x0 => {
                // DltTypeLog - extract log level from bits 4-7
                message_type = "LOG".to_string();
                let log_level = (msin >> 4) & 0x0F;
                log_level_name = match log_level {
                    1 => "FATAL".to_string(),
                    2 => "ERROR".to_string(),
                    3 => "WARN ".to_string(),
                    4 => "INFO ".to_string(),
                    5 => "DEBUG".to_string(),
                    6 => "VERB ".to_string(),
                    _ => format!("LVL{}", log_level),
                };
            },
            0x1 => {
                message_type = "APP_TRACE".to_string();
            },
            0x2 => {
                message_type = "NW_TRACE".to_string();
            },
            0x3 => {
                message_type = "CONTROL".to_string();
            },
            _ => {
                message_type = format!("MSTP_{}", mstp);
            }
        }
        
        if message.len() >= ext_offset + 6 {
            app_id = bytes_to_string(&message[ext_offset + 2..ext_offset + 6]);
        }
        if message.len() >= ext_offset + 10 {
            ctx_id = bytes_to_string(&message[ext_offset + 6..ext_offset + 10]);
        }
    }

    // Extract payload
    let payload_offset = header_len;
    let mut payload_text = String::new();
    
    if payload_len > 0 && message.len() >= payload_offset + payload_len {
        let payload = &message[payload_offset..payload_offset + payload_len];
        
        // Parse service/control messages
        if message_type == "CONTROL" {
            let parser = DltServiceParser::new(payload);
            match parser.parse_service_id() {
                Ok(service_id) => {
                    match service_id {
                        ServiceId::GetSoftwareVersion => {
                            match parser.parse_get_software_version_response() {
                                Ok((status, version)) => {
                                    payload_text = format!("GetSoftwareVersion: {} (status: {:?})", 
                                        String::from_utf8_lossy(version),
                                        status);
                                },
                                Err(e) => {
                                    payload_text = format!("GetSoftwareVersion (parse error: {:?}, payload {} bytes)", 
                                        e, payload.len());
                                }
                            }
                        },
                        ServiceId::SetLogLevel => {
                            if let Ok(status) = parser.parse_status_response() {
                                payload_text = format!("SetLogLevel (status: {:?})", status);
                            }
                        },
                        ServiceId::GetDefaultLogLevel => {
                            if let Ok((status, level)) = parser.parse_get_default_log_level_response() {
                                payload_text = format!("GetDefaultLogLevel: {} (status: {:?})", level, status);
                            }
                        },
                        ServiceId::GetLogInfo => {
                            match parser.parse_get_log_info_response() {
                                Ok((status, log_info_data)) => {
                                    let with_desc = matches!(status, ServiceStatus::WithDescriptions);
                                    let mut info_parser = LogInfoResponseParser::new(log_info_data, with_desc);
                                    
                                    match info_parser.read_app_count() {
                                        Ok(app_count) => {
                                            payload_text = format!("GetLogInfo (status: {:?}): {} app(s)", status, app_count);
                                            
                                            // Parse and display apps/contexts
                                            for app_idx in 0..app_count {
                                                if let Ok(app_id) = info_parser.read_app_id() {
                                                    let app_str = String::from_utf8_lossy(&app_id).trim_end_matches('\0').to_string();
                                                    if let Ok(ctx_count) = info_parser.read_context_count() {
                                                        payload_text.push_str(&format!("\n       App[{}]: {:?} ({} contexts)", app_idx, app_str, ctx_count));
                                                        
                                                        for ctx_idx in 0..ctx_count {
                                                            if let Ok((ctx_id, log_lvl, trace)) = info_parser.read_context_info() {
                                                                let ctx_str = String::from_utf8_lossy(&ctx_id).trim_end_matches('\0').to_string();
                                                                payload_text.push_str(&format!("\n         Ctx[{}]: {:?} [level={}, trace={}]", 
                                                                    ctx_idx, ctx_str, log_lvl, trace));
                                                                
                                                                if with_desc {
                                                                    if let Ok(desc) = info_parser.read_description() {
                                                                        if !desc.is_empty() {
                                                                            payload_text.push_str(&format!(" - {}", String::from_utf8_lossy(desc)));
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        
                                                        if with_desc {
                                                            if let Ok(app_desc) = info_parser.read_description() {
                                                                if !app_desc.is_empty() {
                                                                    payload_text.push_str(&format!("\n       AppDesc: {}", String::from_utf8_lossy(app_desc)));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            payload_text = format!("GetLogInfo (parse error: {:?})", e);
                                        }
                                    }
                                },
                                Err(e) => {
                                    payload_text = format!("GetLogInfo (parse error: {:?})", e);
                                }
                            }
                        },
                        _ => {
                            payload_text = format!("{:?}", service_id);
                        }
                    }
                },
                Err(e) => {
                    payload_text = format!("Service parse error: {:?}", e);
                }
            }
            
            // Fallback to hex if payload_text is still empty
            if payload_text.is_empty() {
                payload_text = payload.iter()
                    .take(32)
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                if payload_len > 32 {
                    payload_text.push_str("...");
                }
            }
        } else {
            // Try to extract text (filter printable ASCII)
            payload_text = payload.iter()
                .filter(|&&b| b >= 32 && b < 127)
                .map(|&b| b as char)
                .collect();
            
            // If too few printable chars, show hex instead
            if payload_text.len() < payload_len / 2 {
                payload_text = payload.iter()
                    .take(32)
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                if payload_len > 32 {
                    payload_text.push_str("...");
                }
            }
        }
    }

    // Display the message
    let type_display = if !log_level_name.is_empty() {
        format!("{} {}", message_type, log_level_name)
    } else {
        message_type
    };
    
    println!("\n#{} | {} | ECU:{:4} | App:{:4} | Ctx:{:4} | {} bytes",
        msg_num,
        type_display,
        ecu_id,
        app_id,
        ctx_id,
        total_len
    );
    
    if !payload_text.is_empty() {
        println!("    └─ {}", payload_text);
    }
}

fn bytes_to_string(bytes: &[u8]) -> String {
    bytes.iter()
        .filter(|&&b| b != 0)
        .map(|&b| b as char)
        .collect()
}
