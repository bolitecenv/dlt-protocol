use dlt_protocol::r19_11::*;

#[test]
fn test_generate_log_message() {
    let mut builder = DltMessageBuilder::new()
        .with_ecu_id(*b"TEST")
        .with_app_id(*b"MYAP")
        .with_context_id(*b"MYCT");

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
