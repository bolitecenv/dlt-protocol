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
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                eprintln!("❌ Error reading standard header: {}", e);
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
    let mut log_level_name = String::from("Unknown");
    let mut app_id = String::new();
    let mut ctx_id = String::new();
    
    if has_extended && message.len() >= ext_offset + 10 {
        let msin = message[ext_offset];
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
        
        if message.len() >= ext_offset + 8 {
            app_id = bytes_to_string(&message[ext_offset + 4..ext_offset + 8]);
        }
        if message.len() >= ext_offset + 12 {
            ctx_id = bytes_to_string(&message[ext_offset + 8..ext_offset + 12]);
        }
    }

    // Extract payload
    let payload_offset = header_len;
    let mut payload_text = String::new();
    
    if payload_len > 0 && message.len() >= payload_offset + payload_len {
        let payload = &message[payload_offset..payload_offset + payload_len];
        
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

    // Display the message
    println!("\n#{} | {} | ECU:{:4} | App:{:4} | Ctx:{:4} | {} bytes",
        msg_num,
        log_level_name,
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
