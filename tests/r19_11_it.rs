use dlt_protocol::r19_11::*;

#[test]
fn test_generate_log_message() {
    let mut builder = DltMessageBuilder::new()
        .with_ecu_id(b"TEST")
        .with_app_id(b"MYAP")
        .with_context_id(b"MYCT");

    let mut buffer = [0u8; 256];
    let text = b"Hello, DLT!";

    let result = builder.log_text(&mut buffer, MtinTypeDltLog::DltLogInfo, text);
    assert!(result.is_ok());

    let len = result.unwrap();
    assert!(len > 0);
    assert_eq!(buffer[0] & UEH_MASK, UEH_MASK); // Extended header present
    assert_eq!(buffer[1], 0); // First message counter
    assert_eq!(u16::from_be_bytes([buffer[2], buffer[3]]), len as u16); // Length
    assert_eq!(&buffer[4..8], b"TEST"); // ECU ID
    assert_eq!(&buffer[18..22], b"MYAP"); // App ID
    assert_eq!(&buffer[22..26], b"MYCT"); // Context ID
}

#[test]
fn test_message_counter_increment() {
    let mut builder = DltMessageBuilder::new();
    let mut buffer = [0u8; 256];
    let text = b"Test";

    builder
        .log_text(&mut buffer, MtinTypeDltLog::DltLogInfo, text)
        .unwrap();
    assert_eq!(buffer[1], 0);

    builder
        .log_text(&mut buffer, MtinTypeDltLog::DltLogInfo, text)
        .unwrap();
    assert_eq!(buffer[1], 1);

    builder
        .log_text(&mut buffer, MtinTypeDltLog::DltLogInfo, text)
        .unwrap();
    assert_eq!(buffer[1], 2);
}

#[test]
fn test_buffer_too_small() {
    let mut builder = DltMessageBuilder::new();
    let mut buffer = [0u8; 10]; // Too small
    let text = b"Hello, DLT!";

    let result = builder.log_text(&mut buffer, MtinTypeDltLog::DltLogInfo, text);
    assert_eq!(result, Err(DltError::BufferTooSmall));
}

#[test]
fn test_payload_builder() {
    let mut buffer = [0u8; 256];
    let mut builder = PayloadBuilder::new(&mut buffer);

    builder.add_u32(0x12345678).unwrap();
    builder.add_string("Hello").unwrap();
    builder.add_bool(true).unwrap();

    assert!(builder.len() > 0);

    // Verify first type info (u32)
    let slice = builder.as_slice();
    let first_type_info = u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]);
    assert_eq!(first_type_info & 0x0F, 0x03); // TYLE = 32 bit
    assert_eq!(first_type_info & (1 << 6), 1 << 6); // UINT bit
}

#[test]
fn test_parse_and_build() {
    let mut buffer = [0u8; 256];
    let mut builder = PayloadBuilder::new(&mut buffer);

    builder.add_u32(42).unwrap();
    builder.add_string("Test").unwrap();

    let payload = builder.as_slice();
    let mut parser = PayloadParser::new(payload);

    assert_eq!(parser.read_u32().unwrap(), 42);
    assert_eq!(parser.read_string().unwrap(), "Test");
}
