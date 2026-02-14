/// # Simple DLT Daemon Example
///
/// A minimal DLT daemon demonstrating the DLT R19-11 protocol library:
/// - Opens a TCP server on localhost:3490
/// - Accepts client connections  
/// - Parses DLT messages using `DltHeaderParser`
/// - Parses service requests using `DltServiceParser`
/// - Generates service responses using `DltServiceMessageBuilder`
/// - Generates log messages using `DltMessageBuilder`
/// - Parses verbose payloads using `PayloadParser`
/// - Sends periodic log messages
///
/// ## Usage
///
/// Start the daemon:
/// ```bash
/// cargo run --example dlt_daemon_simple
/// ```
///
/// Connect with the viewer:
/// ```bash
/// cargo run --example dlt_console_viewer
/// ```

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dlt_protocol::r19_11::*;

// Global daemon state
struct DaemonState {
    default_log_level: i8,
    message_filtering_enabled: bool,
    session_counter: u32,
    software_version: String,
}

impl DaemonState {
    fn new() -> Self {
        Self {
            default_log_level: MtinTypeDltLog::DltLogInfo.to_bits() as i8,
            message_filtering_enabled: false,
            session_counter: 1,
            software_version: "1.0.0".to_string(),
        }
    }
}

fn main() -> std::io::Result<()> {
    println!("🚀 DLT Daemon - Simple Example");
    println!("Listening on localhost:3490...\n");

    let listener = TcpListener::bind("127.0.0.1:3490")?;
    let state = Arc::new(Mutex::new(DaemonState::new()));

    println!("✅ DLT Daemon started successfully!");
    println!("{}", "=".repeat(80));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state_clone = Arc::clone(&state);
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream, state_clone) {
                        eprintln!("❌ Client handler error: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("❌ Connection error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, state: Arc<Mutex<DaemonState>>) -> std::io::Result<()> {
    let peer_addr = stream.peer_addr()?;
    println!("📡 New connection from: {}", peer_addr);

    // Send welcome log message
    if let Err(e) = send_log_message(&mut stream, "DLT Daemon Ready", MtinTypeDltLog::DltLogInfo) {
        eprintln!("⚠️  Failed to send welcome message: {}", e);
        return Err(e);
    }

    let mut buffer = [0u8; 4096];
    
    // Send software version announcement as a service message
    let version = state.lock().unwrap().software_version.clone();
    if let Err(e) = send_software_version_announcement(&mut stream, &mut buffer, &version) {
        eprintln!("⚠️  Failed to send version announcement: {}", e);
        return Err(e);
    }

    let mut read_buffer = [0u8; 4096];
    let mut read_pos = 0;

    // Start periodic log sender
    let mut stream_clone = stream.try_clone()?;
    let log_thread = thread::spawn(move || {
        let mut counter = 0u32;
        loop {
            thread::sleep(Duration::from_secs(5));
            counter += 1;
            let msg = format!("Periodic heartbeat #{}", counter);
            if let Err(_) = send_log_message(&mut stream_clone, &msg, MtinTypeDltLog::DltLogDebug) {
                break;
            }
        }
    });

    loop {
        // Read data from client
        match stream.read(&mut read_buffer[read_pos..]) {
            Ok(0) => {
                println!("📡 Client {} disconnected", peer_addr);
                break;
            }
            Ok(n) => {
                read_pos += n;

                // Try to parse complete messages
                while read_pos >= 4 {
                    // Parse standard header to get message length
                    let std_header = &read_buffer[..4];
                    let msg_len = u16::from_be_bytes([std_header[2], std_header[3]]) as usize;

                    if msg_len < 4 || msg_len > 4096 {
                        eprintln!("⚠️  Invalid message length: {}", msg_len);
                        read_pos = 0;
                        break;
                    }

                    // Wait for complete message
                    if read_pos < msg_len {
                        break;
                    }

                    // Parse the complete message
                    let message_data = &read_buffer[..msg_len];
                    
                    process_message(message_data, &mut stream, &state, &mut buffer);

                    // Shift remaining data
                    read_buffer.copy_within(msg_len..read_pos, 0);
                    read_pos -= msg_len;
                }
            }
            Err(e) => {
                eprintln!("❌ Read error from {}: {}", peer_addr, e);
                break;
            }
        }
    }

    drop(log_thread);
    Ok(())
}

fn process_message(
    data: &[u8],
    stream: &mut TcpStream,
    state: &Arc<Mutex<DaemonState>>,
    buffer: &mut [u8; 4096],
) {
    // Parse the DLT message
    let mut parser = DltHeaderParser::new(data);
    let message = match parser.parse_message() {
        Ok(msg) => msg,
        Err(e) => {
            eprintln!("❌ Parse error: {:?}", e);
            return;
        }
    };

    // Check if it's a service/control message
    if let Some(ext_hdr) = &message.extended_header {
        match ext_hdr.message_type() {
            MstpType::DltTypeControl => {
                let app_id = bytes_to_string(&ext_hdr.apid);
                let ctx_id = bytes_to_string(&ext_hdr.ctid);
                
                // Debug: show raw payload
                println!("📦 Control message received, payload size: {} bytes", message.payload.len());
                if message.payload.len() >= 4 {
                    println!("   Last 4 bytes: {:02x} {:02x} {:02x} {:02x}", 
                        message.payload[message.payload.len()-4], message.payload[message.payload.len()-3],
                        message.payload[message.payload.len()-2], message.payload[message.payload.len()-1]);
                }
                if message.payload.len() <= 32 {
                    print!("   Full payload: ");
                    for b in message.payload {
                        print!("{:02x} ", b);
                    }
                    println!();
                }
                
                // Parse service message
                let service_parser = DltServiceParser::new(message.payload);
                println!("   After DltServiceParser::new(), payload size: {} bytes", service_parser.get_payload().len());
                
                match service_parser.parse_service_id() {
                    Ok(service_id) => {
                        println!("🔧 Service Request: {:?} from {}:{}", 
                                 service_id, app_id, ctx_id);
                        
                        handle_service_request(
                            service_id,
                            &service_parser,
                            stream,
                            state,
                            buffer,
                            &ext_hdr.apid,
                            &ext_hdr.ctid,
                        );
                    }
                    Err(_) => {
                        eprintln!("⚠️  Unknown service ID in message");
                    }
                }
            }
            MstpType::DltTypeLog => {
                // Parse and display log messages
                let app_id = bytes_to_string(&ext_hdr.apid);
                let ctx_id = bytes_to_string(&ext_hdr.ctid);
                let log_level = ext_hdr.log_level()
                    .map(|l| format!("{:?}", l))
                    .unwrap_or_else(|| "Unknown".to_string());
                
                println!("📝 Log message from {}:{} [{}]", app_id, ctx_id, log_level);
                
                // Try to parse verbose payload if present
                if ext_hdr.is_verbose() && !message.payload.is_empty() {
                    let mut payload_parser = PayloadParser::new(message.payload);
                    
                    // Print all arguments in the payload
                    let mut arg_count = 0;
                    while !payload_parser.is_empty() {
                        match payload_parser.read_next() {
                            Ok(value) => {
                                println!("   [arg{}] {:?}", arg_count, value);
                                arg_count += 1;
                            }
                            Err(e) => {
                                eprintln!("   ⚠️  Payload parse error: {:?}", e);
                                break;
                            }
                        }
                    }
                } else if !message.payload.is_empty() {
                    // Non-verbose: just show hex dump
                    print!("   Payload (hex): ");
                    for byte in message.payload.iter().take(32) {
                        print!("{:02x} ", byte);
                    }
                    if message.payload.len() > 32 {
                        print!("... ({} bytes total)", message.payload.len());
                    }
                    println!();
                }
            }
            _ => {
                println!("📦 Other message type: {:?}", ext_hdr.message_type());
            }
        }
    }
}

fn handle_service_request(
    service_id: ServiceId,
    parser: &DltServiceParser,
    stream: &mut TcpStream,
    state: &Arc<Mutex<DaemonState>>,
    buffer: &mut [u8; 4096],
    app_id: &[u8; 4],
    ctx_id: &[u8; 4],
) {
    match service_id {
        ServiceId::SetLogLevel => {
            if let Ok((req_app, req_ctx, level)) = parser.parse_set_log_level_request() {
                println!("  → SetLogLevel: {:?}:{:?} = {}", 
                         bytes_to_string(&req_app), bytes_to_string(&req_ctx), level);
                let _ = send_service_response(stream, buffer, ServiceId::SetLogLevel, ServiceStatus::Ok, app_id, ctx_id);
            }
        }
        
        ServiceId::SetDefaultLogLevel => {
            if let Ok(level) = parser.parse_set_default_log_level_request() {
                println!("  → SetDefaultLogLevel: {}", level);
                state.lock().unwrap().default_log_level = level;
                let _ = send_service_response(stream, buffer, ServiceId::SetDefaultLogLevel, ServiceStatus::Ok, app_id, ctx_id);
            }
        }

        ServiceId::GetDefaultLogLevel => {
            let level = state.lock().unwrap().default_log_level;
            println!("  → GetDefaultLogLevel: {}", level);
            let _ = send_get_default_log_level_response(stream, buffer, level as u8, app_id, ctx_id);
        }

        ServiceId::GetSoftwareVersion => {
            let version = state.lock().unwrap().software_version.clone();
            println!("  → GetSoftwareVersion: {}", version);
            let _ = send_get_software_version_response(stream, buffer, &version, app_id, ctx_id);
        }

        ServiceId::GetLogInfo => {
            println!("  → GetLogInfo request received");
            match parser.parse_get_log_info_request() {
                Ok((options, req_app, req_ctx)) => {
                    let app_str = bytes_to_string(&req_app);
                    let ctx_str = bytes_to_string(&req_ctx);
                    let with_descriptions = options == 7;
                    println!("    Parsed: options={}, app={:?}, ctx={:?}", options, app_str, ctx_str);
                    match send_get_log_info_response(stream, buffer, with_descriptions, &req_app, &req_ctx, app_id, ctx_id) {
                        Ok(_) => println!("    ✓ Response sent successfully"),
                        Err(e) => {
                            println!("    ✗ Failed to send response: {:?}", e);
                            let _ = send_service_response(stream, buffer, ServiceId::GetLogInfo, ServiceStatus::Error, app_id, ctx_id);
                        }
                    }
                }
                Err(e) => {
                    println!("    ✗ Failed to parse request: {:?}", e);
                    let _ = send_service_response(stream, buffer, ServiceId::GetLogInfo, ServiceStatus::Error, app_id, ctx_id);
                }
            }
        }

        ServiceId::SetMessageFiltering => {
            if let Ok(enabled) = parser.parse_set_message_filtering_request() {
                println!("  → SetMessageFiltering: {}", enabled);
                state.lock().unwrap().message_filtering_enabled = enabled;
                let _ = send_service_response(stream, buffer, ServiceId::SetMessageFiltering, ServiceStatus::Ok, app_id, ctx_id);
            }
        }

        ServiceId::StoreConfiguration => {
            println!("  → StoreConfiguration");
            let _ = send_service_response(stream, buffer, ServiceId::StoreConfiguration, ServiceStatus::Ok, app_id, ctx_id);
            let _ = send_log_message(stream, "Configuration stored", MtinTypeDltLog::DltLogInfo);
        }

        ServiceId::ResetToFactoryDefault => {
            println!("  → ResetToFactoryDefault");
            *state.lock().unwrap() = DaemonState::new();
            let _ = send_service_response(stream, buffer, ServiceId::ResetToFactoryDefault, ServiceStatus::Ok, app_id, ctx_id);
            let _ = send_log_message(stream, "Reset to factory defaults", MtinTypeDltLog::DltLogWarn);
        }

        _ => {
            println!("  → Unsupported service: {:?}", service_id);
            let _ = send_service_response(stream, buffer, service_id, ServiceStatus::NotSupported, app_id, ctx_id);
        }
    }
}

fn send_service_response(
    stream: &mut TcpStream,
    buffer: &mut [u8; 4096],
    service_id: ServiceId,
    status: ServiceStatus,
    app_id: &[u8; 4],
    ctx_id: &[u8; 4],
) -> Result<(), DltError> {
    let mut builder = DltServiceMessageBuilder::new()
        .with_ecu_id(b"DMND")
        .with_app_id(app_id)
        .with_context_id(ctx_id);

    let len = builder.generate_status_response(buffer, service_id, status)?;
    
    stream.write_all(&buffer[..len])
        .map_err(|_| DltError::BufferTooSmall)?;
    
    Ok(())
}

fn send_get_default_log_level_response(
    stream: &mut TcpStream,
    buffer: &mut [u8; 4096],
    log_level: u8,
    app_id: &[u8; 4],
    ctx_id: &[u8; 4],
) -> Result<(), DltError> {
    let mut builder = DltServiceMessageBuilder::new()
        .with_ecu_id(b"DMND")
        .with_app_id(app_id)
        .with_context_id(ctx_id);

    let len = builder.generate_get_default_log_level_response(buffer, ServiceStatus::Ok, log_level)?;
    
    stream.write_all(&buffer[..len])
        .map_err(|_| DltError::BufferTooSmall)?;
    
    Ok(())
}

fn send_get_software_version_response(
    stream: &mut TcpStream,
    buffer: &mut [u8; 4096],
    version: &str,
    app_id: &[u8; 4],
    ctx_id: &[u8; 4],
) -> Result<(), DltError> {
    let mut builder = DltServiceMessageBuilder::new()
        .with_ecu_id(b"DMND")
        .with_app_id(app_id)
        .with_context_id(ctx_id);

    let len = builder.generate_get_software_version_response(buffer, ServiceStatus::Ok, version.as_bytes())?;
    
    stream.write_all(&buffer[..len])
        .map_err(|_| DltError::BufferTooSmall)?;
    
    Ok(())
}

fn send_get_log_info_response(
    stream: &mut TcpStream,
    buffer: &mut [u8; 4096],
    with_descriptions: bool,
    req_app_id: &[u8; 4],
    req_ctx_id: &[u8; 4],
    app_id: &[u8; 4],
    ctx_id: &[u8; 4],
) -> Result<(), DltError> {
    // Build the log info payload based on what the daemon knows
    // In a real daemon, this would query registered applications and contexts
    let mut payload_buffer = [0u8; 2048];
    let mut writer = LogInfoPayloadWriter::new(&mut payload_buffer, with_descriptions);

    // Check if filtering by specific app/context or return all (wildcard)
    let is_wildcard_app = is_wildcard_id(req_app_id);
    let is_wildcard_ctx = is_wildcard_id(req_ctx_id);

    // For this simple example, we have one app (DMND) with one context (CORE)
    // In a real daemon, you'd iterate through all registered apps/contexts
    let should_include_dmnd = is_wildcard_app || req_app_id == b"DMND";
    
    if should_include_dmnd {
        let dmnd_contexts: Vec<(&[u8], u8, u8, Option<&[u8]>)> = if is_wildcard_ctx || req_ctx_id == b"CORE" {
            vec![
                (b"CORE", 4, 1, if with_descriptions { Some(b"Daemon core context") } else { None }),
            ]
        } else {
            vec![]
        };

        if !dmnd_contexts.is_empty() {
            writer.write_app_count(1)?;
            writer.write_app_id(b"DMND")?;
            writer.write_context_count(dmnd_contexts.len() as u16)?;
            
            for (ctx_id_val, log_level, trace_status, desc) in dmnd_contexts {
                writer.write_context(ctx_id_val, log_level, trace_status, desc)?;
            }
            
            if with_descriptions {
                writer.write_app_description(Some(b"DLT Daemon application"))?;
            }
        } else {
            // No matching contexts
            writer.write_app_count(0)?;
        }
    } else {
        // No matching apps
        writer.write_app_count(0)?;
    }

    let payload_len = writer.finish()?;

    // Generate the complete service message
    let mut builder = DltServiceMessageBuilder::new()
        .with_ecu_id(b"DMND")
        .with_app_id(app_id)
        .with_context_id(ctx_id);

    let status = if with_descriptions {
        ServiceStatus::WithDescriptions
    } else {
        ServiceStatus::WithLogLevelAndTraceStatus
    };

    let len = builder.generate_get_log_info_response(
        buffer,
        status,
        &payload_buffer[..payload_len]
    )?;
    
    stream.write_all(&buffer[..len])
        .map_err(|_| DltError::BufferTooSmall)?;
    
    Ok(())
}

fn send_software_version_announcement(
    stream: &mut TcpStream,
    buffer: &mut [u8; 4096],
    version: &str,
) -> std::io::Result<()> {
    let mut builder = DltServiceMessageBuilder::new()
        .with_ecu_id(b"DMND")
        .with_app_id(b"DMND")
        .with_context_id(b"CORE");

    match builder.generate_get_software_version_response(buffer, ServiceStatus::Ok, version.as_bytes()) {
        Ok(len) => {
            stream.write_all(&buffer[..len])?;
            stream.flush()?;
            Ok(())
        }
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to generate software version message",
        )),
    }
}

fn send_log_message(
    stream: &mut TcpStream,
    message: &str,
    log_level: MtinTypeDltLog,
) -> std::io::Result<()> {
    let mut buffer = [0u8; 512];
    let mut builder = DltMessageBuilder::new()
        .with_ecu_id(b"DMND")
        .with_app_id(b"DMND")
        .with_context_id(b"CORE")
        .with_timestamp(get_timestamp());

    match builder.generate_log_message_with_payload(
        &mut buffer,
        message.as_bytes(),
        log_level,
        1,
        true, // verbose mode - uses PayloadBuilder to encode typed payload
    ) {
        Ok(len) => {
            stream.write_all(&buffer[..len])?;
            stream.flush()?;
            Ok(())
        }
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to generate log message",
        )),
    }
}

fn get_timestamp() -> u32 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    (now.as_millis() % (u32::MAX as u128)) as u32
}

fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .trim_end_matches('\0')
        .to_string()
}
