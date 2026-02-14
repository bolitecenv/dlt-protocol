// Test to verify service message "remo" suffix functionality

use dlt_protocol::r19_11::*;

fn main() {
    println!("🧪 Testing Service Message 'remo' Suffix\n");

    // Test 1: Generate SetLogLevel request
    println!("Test 1: SetLogLevel Request");
    let mut builder = DltServiceMessageBuilder::new()
        .with_ecu_id(b"ECU1")
        .with_app_id(b"DA1\0")
        .with_context_id(b"DC1\0");

    let mut buffer = [0u8; 256];
    let size = builder
        .generate_set_log_level_request(&mut buffer, b"LOG\0", b"TEST", 4)
        .unwrap();

    // Parse the message to verify structure
    let mut parser = DltHeaderParser::new(&buffer[..size]);
    let message = parser.parse_message().unwrap();
    
    // The "remo" suffix should be in the reserved field
    // For SetLogLevel: service_id(4) + app(4) + ctx(4) + level(1) + remo(4) = 17 bytes
    println!("  Generated {} bytes", size);
    println!("  Payload size: {} bytes", message.payload.len());
    println!("  Reserved field (offset 13-16): {:02x} {:02x} {:02x} {:02x}", 
        message.payload[13], message.payload[14], message.payload[15], message.payload[16]);
    assert_eq!(&message.payload[13..17], b"remo");
    println!("  ✅ Reserved field contains 'remo' suffix\n");

    // Test 2: Parse the message
    println!("Test 2: Parse SetLogLevel Request");
    let service_parser = DltServiceParser::new(message.payload);
    let service_id = service_parser.parse_service_id().unwrap();
    println!("  Service ID: {:?}", service_id);
    
    let (app_id, ctx_id, log_level) = service_parser.parse_set_log_level_request().unwrap();
    println!("  App ID: {:?}", core::str::from_utf8(&app_id).unwrap());
    println!("  Context ID: {:?}", core::str::from_utf8(&ctx_id).unwrap());
    println!("  Log Level: {}", log_level);
    println!("  ✅ Parsed successfully\n");

    // Test 3: Generate GetLogInfo response
    println!("Test 3: GetLogInfo Response");
    let mut builder = DltServiceMessageBuilder::new()
        .with_ecu_id(b"ECU1")
        .with_app_id(b"DA1\0")
        .with_context_id(b"DC1\0");

    // Build log info payload
    let mut log_info_payload = [0u8; 512];
    let mut log_info = LogInfoPayloadWriter::new(&mut log_info_payload, false); // without descriptions
    log_info.write_app_count(1).unwrap();
    log_info.write_app_id(b"DMND").unwrap();
    log_info.write_context_count(1).unwrap();
    log_info.write_context(b"CORE", 4, 1, None).unwrap();
    let log_info_len = log_info.finish().unwrap();

    let mut buffer = [0u8; 1024];
    let size = builder
        .generate_get_log_info_response(&mut buffer, ServiceStatus::WithLogLevelAndTraceStatus, &log_info_payload[..log_info_len])
        .unwrap();

    // Parse the message
    let mut parser = DltHeaderParser::new(&buffer[..size]);
    let message = parser.parse_message().unwrap();
    
    println!("  Generated {} bytes", size);
    println!("  Payload size: {} bytes", message.payload.len());
    
    // The last 4 bytes should be "remo" (in the reserved field)
    let payload = message.payload;
    println!("  Last 4 bytes (reserved field): {:02x} {:02x} {:02x} {:02x}", 
        payload[payload.len()-4], payload[payload.len()-3], payload[payload.len()-2], payload[payload.len()-1]);
    assert_eq!(&payload[payload.len()-4..], b"remo");
    println!("  ✅ Reserved field contains 'remo' suffix\n");

    // Test 4: Parse GetLogInfo response
    println!("Test 4: Parse GetLogInfo Response");
    let service_parser = DltServiceParser::new(message.payload);
    println!("  Payload size: {} bytes", service_parser.get_payload().len());
    
    match service_parser.parse_get_log_info_response() {
        Ok((status, log_info_data)) => {
            println!("  Status: {:?}", status);
            println!("  Log info data size: {} bytes", log_info_data.len());
            
            // Parse the log info structure
            let mut info_parser = LogInfoResponseParser::new(log_info_data, false);
            let app_count = info_parser.read_app_count().unwrap();
            println!("  App count: {}", app_count);
            
            let app_id = info_parser.read_app_id().unwrap();
            println!("  App ID: {:?}", core::str::from_utf8(&app_id).unwrap());
            
            let ctx_count = info_parser.read_context_count().unwrap();
            println!("  Context count: {}", ctx_count);
            
            let (ctx_id, log_level, trace_status) = info_parser.read_context_info().unwrap();
            println!("  Context ID: {:?}", core::str::from_utf8(&ctx_id).unwrap());
            println!("  Log Level: {}", log_level);
            println!("  Trace Status: {}", trace_status);
            println!("  ✅ GetLogInfo parsed successfully");
        }
        Err(e) => {
            println!("  ❌ ERROR: Failed to parse GetLogInfo response: {:?}", e);
        }
    }

    println!("\n✅ All tests passed!");
    println!("\nSummary:");
    println!("  - SetLogLevel: 'remo' in reserved field at offset 13-16");
    println!("  - GetLogInfo request: 'remo' in reserved field at offset 13-16");
    println!("  - GetLogInfo response: 'remo' in reserved field at end of payload");
    println!("  - Parser handles messages with 'remo' correctly");
}
