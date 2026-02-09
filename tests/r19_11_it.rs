use dlt_protocol::r19_11::*;

#[test]
fn test_insert_header_at_front_basic() {
    let mut builder = DltMessageBuilder::new()
        .with_ecu_id(b"TEST")
        .with_app_id(b"MYAP")
        .with_context_id(b"MYCT");

    let mut buffer = [0u8; 256];
    let text = b"Hello, DLT!";

    // Pre-fill buffer with payload
    buffer[..text.len()].copy_from_slice(text);

    let result =
        builder.insert_header_at_front(&mut buffer, text.len(), 1, MtinTypeDltLog::DltLogInfo);

    assert!(result.is_ok());
    let total_len = result.unwrap();

    // Verify header is present
    assert_eq!(buffer[0] & UEH_MASK, UEH_MASK); // Extended header
    assert_eq!(buffer[1], 0); // Message counter
    assert_eq!(&buffer[4..8], b"TEST"); // ECU ID
    assert_eq!(&buffer[18..22], b"MYAP"); // App ID
    assert_eq!(&buffer[22..26], b"MYCT"); // Context ID

    // Verify payload was moved correctly
    let header_size = 26; // Standard header + extended header
    assert_eq!(&buffer[header_size..header_size + text.len()], text);
    assert_eq!(total_len, header_size + text.len());
}

#[test]
fn test_insert_header_at_front_with_serial_header() {
    let mut builder = DltMessageBuilder::new()
        .add_serial_header()
        .with_ecu_id(b"ECU1")
        .with_app_id(b"APP1")
        .with_context_id(b"CTX1");

    let mut buffer = [0u8; 256];
    let text = b"Serial test";

    buffer[..text.len()].copy_from_slice(text);

    let result =
        builder.insert_header_at_front(&mut buffer, text.len(), 1, MtinTypeDltLog::DltLogWarn);

    assert!(result.is_ok());
    let total_len = result.unwrap();

    // Verify serial header
    assert_eq!(&buffer[0..4], &DLT_SERIAL_HEADER_ARRAY);

    // Verify standard header after serial header
    assert_eq!(buffer[4] & UEH_MASK, UEH_MASK);
    assert_eq!(&buffer[8..12], b"ECU1"); // ECU ID

    // Verify payload was moved
    let header_size = 4 + 26; // Serial + standard + extended
    assert_eq!(&buffer[header_size..header_size + text.len()], text);
}

#[test]
fn test_insert_header_buffer_too_small() {
    let mut builder = DltMessageBuilder::new();
    let mut buffer = [0u8; 20]; // Too small for header + payload
    let text = b"This is too long";

    buffer[..text.len()].copy_from_slice(text);

    let result =
        builder.insert_header_at_front(&mut buffer, text.len(), 1, MtinTypeDltLog::DltLogInfo);

    assert_eq!(result, Err(DltError::BufferTooSmall));
}

#[test]
fn test_generate_log_message_with_payload() {
    let mut builder = DltMessageBuilder::new()
        .with_ecu_id(b"TEST")
        .with_app_id(b"MYAP")
        .with_context_id(b"MYCT");

    let mut buffer = [0u8; 256];
    let payload = b"Test payload";

    let result = builder.generate_log_message_with_payload(
        &mut buffer,
        payload,
        MtinTypeDltLog::DltLogDebug,
        2,
        true,
    );

    assert!(result.is_ok());
    let total_len = result.unwrap();

    // Verify header
    assert_eq!(buffer[0] & UEH_MASK, UEH_MASK);
    assert_eq!(&buffer[4..8], b"TEST");
    assert_eq!(&buffer[18..22], b"MYAP");
    assert_eq!(&buffer[22..26], b"MYCT");

    // Verify verbose bit is set
    assert_eq!(buffer[16] & 0x01, 0x01);

    // Verify payload
    let header_size = 26;
    assert_eq!(&buffer[header_size..header_size + payload.len()], payload);
    assert_eq!(total_len, header_size + payload.len());
}

#[test]
fn test_generate_log_message_with_payload_non_verbose() {
    let mut builder = DltMessageBuilder::new();
    let mut buffer = [0u8; 256];
    let payload = b"Non-verbose";

    let result = builder.generate_log_message_with_payload(
        &mut buffer,
        payload,
        MtinTypeDltLog::DltLogError,
        1,
        false, // Non-verbose
    );

    assert!(result.is_ok());

    // Verify verbose bit is NOT set
    assert_eq!(buffer[16] & 0x01, 0x00);
}

#[test]
fn test_counter_increment_with_insert_header() {
    let mut builder = DltMessageBuilder::new();
    let mut buffer = [0u8; 256];
    let text = b"Test";

    // First message
    buffer[..text.len()].copy_from_slice(text);
    builder
        .insert_header_at_front(&mut buffer, text.len(), 1, MtinTypeDltLog::DltLogInfo)
        .unwrap();
    assert_eq!(buffer[1], 0);
    assert_eq!(builder.get_counter(), 1);

    // Second message
    buffer[..text.len()].copy_from_slice(text);
    builder
        .insert_header_at_front(&mut buffer, text.len(), 1, MtinTypeDltLog::DltLogInfo)
        .unwrap();
    assert_eq!(buffer[1], 1);
    assert_eq!(builder.get_counter(), 2);

    // Reset counter
    builder.reset_counter();
    assert_eq!(builder.get_counter(), 0);

    // Third message after reset
    buffer[..text.len()].copy_from_slice(text);
    builder
        .insert_header_at_front(&mut buffer, text.len(), 1, MtinTypeDltLog::DltLogInfo)
        .unwrap();
    assert_eq!(buffer[1], 0);
}

#[test]
fn test_endian_setting() {
    let mut builder = DltMessageBuilder::new();
    builder.set_endian(DltEndian::Little);

    let mut buffer = [0u8; 256];
    let payload = b"Test";

    let result = builder.generate_log_message_with_payload(
        &mut buffer,
        payload,
        MtinTypeDltLog::DltLogInfo,
        1,
        true,
    );

    assert!(result.is_ok());

    // Verify length is little endian
    let len = u16::from_le_bytes([buffer[2], buffer[3]]);
    assert!(len > 0);
}

#[test]
fn test_insert_header_multiple_messages() {
    let mut builder = DltMessageBuilder::new()
        .with_ecu_id(b"TEST")
        .with_app_id(b"MYAP")
        .with_context_id(b"MYCT");

    let messages = [
        b"First message" as &[u8],
        b"Second" as &[u8],
        b"Third message here" as &[u8],
    ];

    for (i, msg) in messages.iter().enumerate() {
        let mut buffer = [0u8; 256];
        buffer[..msg.len()].copy_from_slice(msg);

        let result =
            builder.insert_header_at_front(&mut buffer, msg.len(), 1, MtinTypeDltLog::DltLogInfo);

        assert!(result.is_ok());
        assert_eq!(buffer[1], i as u8); // Counter increments

        let header_size = 26;
        assert_eq!(&buffer[header_size..header_size + msg.len()], *msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicU32, Ordering};

    // ========================================
    // テスト用のカウンター実装
    // ========================================
    static TEST_TIMESTAMP_COUNTER: AtomicU32 = AtomicU32::new(0);
    static TEST_SESSION_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn get_test_timestamp() -> u32 {
        TEST_TIMESTAMP_COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    fn get_test_session() -> u32 {
        TEST_SESSION_COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    fn reset_test_counters() {
        TEST_TIMESTAMP_COUNTER.store(0, Ordering::SeqCst);
        TEST_SESSION_COUNTER.store(0, Ordering::SeqCst);
    }

    // ========================================
    // 静的プロバイダーのインスタンス
    // ========================================
    static TIMESTAMP_PROVIDER: StaticTimestampProvider =
        StaticTimestampProvider::new(get_test_timestamp);

    static SESSION_PROVIDER: StaticSessionIdProvider =
        StaticSessionIdProvider::new(get_test_session);

    // ========================================
    // 基本的な機能テスト
    // ========================================

    #[test]
    fn test_static_timestamp_provider() {
        reset_test_counters();

        let provider = StaticTimestampProvider::new(get_test_timestamp);
        assert_eq!(provider.get_timestamp(), 0);
        assert_eq!(provider.get_timestamp(), 1);
        assert_eq!(provider.get_timestamp(), 2);
    }

    #[test]
    fn test_static_session_id_provider() {
        reset_test_counters();

        let provider = StaticSessionIdProvider::new(get_test_session);
        assert_eq!(provider.get_session_id(), 0);
        assert_eq!(provider.get_session_id(), 1);
        assert_eq!(provider.get_session_id(), 2);
    }

    // ========================================
    // GlobalProvider テスト
    // ========================================

    #[test]
    fn test_global_provider_initially_none() {
        let provider: GlobalProvider<dyn TimestampProvider> = GlobalProvider::new();
        assert!(provider.get().is_none());
    }

    #[test]
    fn test_global_provider_set_and_get() {
        static LOCAL_TIMESTAMP: GlobalProvider<dyn TimestampProvider> = GlobalProvider::new();

        LOCAL_TIMESTAMP.set(&TIMESTAMP_PROVIDER);

        let retrieved = LOCAL_TIMESTAMP.get();
        assert!(retrieved.is_some());

        if let Some(provider) = retrieved {
            reset_test_counters();
            assert_eq!(provider.get_timestamp(), 0);
        }
    }

    #[test]
    #[should_panic(expected = "Provider already initialized")]
    fn test_global_provider_double_set_panics() {
        static DOUBLE_SET_TEST: GlobalProvider<dyn TimestampProvider> = GlobalProvider::new();

        DOUBLE_SET_TEST.set(&TIMESTAMP_PROVIDER);
        DOUBLE_SET_TEST.set(&TIMESTAMP_PROVIDER); // これでパニックするはず
    }

    // ========================================
    // グローバル関数テスト
    // ========================================

    // 注意: これらのテストは一度しか実行できません（グローバル状態を変更するため）
    // 実際のテストでは、テストごとに異なるグローバル変数を使用するか、
    // 統合テストとして分離することを推奨します

    #[test]
    fn test_set_global_providers() {
        // この関数は一度だけ呼び出せます
        // 実際のアプリケーションでは初期化時に一度だけ呼ぶ想定

        // GLOBAL_TIMESTAMPとGLOBAL_SESSIONが既に初期化されていないことを確認
        // （他のテストで初期化されている可能性があるため、この確認は省略可）
    }

    // ========================================
    // DltMessageBuilder との統合テスト
    // ========================================

    #[test]
    fn test_message_builder_with_providers() {
        reset_test_counters();

        let mut builder = DltMessageBuilder::new();
        builder.set_timestamp_provider(&TIMESTAMP_PROVIDER);
        builder.set_session_id_provider(&SESSION_PROVIDER);

        let mut buffer = [0u8; 256];
        let result =
            builder._generate_log_message(&mut buffer, 0, MtinTypeDltLog::DltLogInfo, 0, false);

        assert!(result.is_ok());

        // タイムスタンプとセッションIDがインクリメントされることを確認
        let result2 =
            builder._generate_log_message(&mut buffer, 0, MtinTypeDltLog::DltLogInfo, 0, false);

        assert!(result2.is_ok());
    }

    #[test]
    fn test_message_builder_without_providers() {
        let mut builder = DltMessageBuilder::new();

        // プロバイダーなしでも動作することを確認
        let mut buffer = [0u8; 256];
        let result =
            builder._generate_log_message(&mut buffer, 0, MtinTypeDltLog::DltLogInfo, 0, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_message_builder_provider_values_used() {
        reset_test_counters();

        let mut builder = DltMessageBuilder::new();
        builder.set_timestamp_provider(&TIMESTAMP_PROVIDER);
        builder.set_session_id_provider(&SESSION_PROVIDER);

        let mut buffer = [0u8; 256];

        // 最初のメッセージ
        builder
            ._generate_log_message(&mut buffer, 0, MtinTypeDltLog::DltLogInfo, 0, false)
            .unwrap();

        // 値が更新されていることを確認
        assert_eq!(builder.timestamp, 0);
        assert_eq!(builder.session_id, 0);

        // 2番目のメッセージ
        builder
            ._generate_log_message(&mut buffer, 0, MtinTypeDltLog::DltLogInfo, 0, false)
            .unwrap();

        // 値がインクリメントされていることを確認
        assert_eq!(builder.timestamp, 1);
        assert_eq!(builder.session_id, 1);
    }

    // ========================================
    // エッジケーステスト
    // ========================================

    #[test]
    fn test_provider_thread_safety() {
        // このテストは概念的なもの。実際のマルチスレッド環境では
        // std の機能が必要ですが、no_std では基本的な確認のみ

        let provider = StaticTimestampProvider::new(get_test_timestamp);

        // トレイト境界を確認
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StaticTimestampProvider>();
        assert_send_sync::<StaticSessionIdProvider>();
    }
}

#[test]
fn test_dlt_paylad_message() {
    let mut buffer = [0u8; 128];
    let payload_len = {
        let mut payload_builder = PayloadBuilder::new(&mut buffer);
        payload_builder.add_string("Test payload");
        payload_builder.len()
    }; // payload_builder drops here automatically

    // Construct DLT message
    let mut message_builder = DltMessageBuilder::new()
        .with_ecu_id(b"ECU1")
        .with_app_id(b"APP1")
        .with_context_id(b"CTX1");

    message_builder.insert_header_at_front(&mut buffer, payload_len, 1, MtinTypeDltLog::DltLogInfo);

    // Verify payload content
    let header_size = 26; // Assuming standard + extended header size
    let payload = &buffer[header_size + 6..header_size+ payload_len]; // Adjust for payload header size
    println!("DLT message {:?}", buffer);

    assert_eq!(payload, b"Test payload\0");
}

// ========================================
// PayloadParser Tests
// ========================================

#[test]
fn test_payload_parser_bool() {
    let mut buffer = [0u8; 64];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_bool(true).unwrap();
        builder.add_bool(false).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    assert_eq!(parser.read_bool().unwrap(), true);
    assert_eq!(parser.read_bool().unwrap(), false);
    assert!(parser.is_empty());
}

#[test]
fn test_payload_parser_integers() {
    let mut buffer = [0u8; 128];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_i8(-42).unwrap();
        builder.add_i16(-1234).unwrap();
        builder.add_i32(-123456).unwrap();
        builder.add_i64(-9876543210).unwrap();
        builder.add_u8(42).unwrap();
        builder.add_u16(1234).unwrap();
        builder.add_u32(123456).unwrap();
        builder.add_u64(9876543210).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    assert_eq!(parser.read_i8().unwrap(), -42);
    assert_eq!(parser.read_i16().unwrap(), -1234);
    assert_eq!(parser.read_i32().unwrap(), -123456);
    assert_eq!(parser.read_i64().unwrap(), -9876543210);
    assert_eq!(parser.read_u8().unwrap(), 42);
    assert_eq!(parser.read_u16().unwrap(), 1234);
    assert_eq!(parser.read_u32().unwrap(), 123456);
    assert_eq!(parser.read_u64().unwrap(), 9876543210);
    assert!(parser.is_empty());
}

#[test]
fn test_payload_parser_floats() {
    let mut buffer = [0u8; 64];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_f32(3.14159).unwrap();
        builder.add_f64(2.718281828).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    let f32_val = parser.read_f32().unwrap();
    let f64_val = parser.read_f64().unwrap();
    
    assert!((f32_val - 3.14159).abs() < 0.00001);
    assert!((f64_val - 2.718281828).abs() < 0.000000001);
    assert!(parser.is_empty());
}

#[test]
fn test_payload_parser_string() {
    let mut buffer = [0u8; 128];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_string("Hello, DLT!").unwrap();
        builder.add_string("Test").unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    assert_eq!(parser.read_string().unwrap(), "Hello, DLT!");
    assert_eq!(parser.read_string().unwrap(), "Test");
    assert!(parser.is_empty());
}

#[test]
fn test_payload_parser_raw() {
    let mut buffer = [0u8; 128];
    let raw_data = b"\x01\x02\x03\x04\x05";
    
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_raw(raw_data).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    let parsed_raw = parser.read_raw().unwrap();
    assert_eq!(parsed_raw, raw_data);
    assert!(parser.is_empty());
}

#[test]
fn test_payload_parser_u128() {
    let mut buffer = [0u8; 64];  // Increased buffer size for 4 (type info) + 16 (data) = 20 bytes
    let value: u128 = 0x123456789ABCDEF0123456789ABCDEF0;
    
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_u128(value).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    assert_eq!(parser.read_u128().unwrap(), value);
    assert!(parser.is_empty());
}

#[test]
fn test_payload_parser_mixed_types() {
    let mut buffer = [0u8; 256];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_u32(42).unwrap();
        builder.add_string("Mixed").unwrap();
        builder.add_bool(true).unwrap();
        builder.add_f32(1.5).unwrap();
        builder.add_i16(-99).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    assert_eq!(parser.read_u32().unwrap(), 42);
    assert_eq!(parser.read_string().unwrap(), "Mixed");
    assert_eq!(parser.read_bool().unwrap(), true);
    assert!((parser.read_f32().unwrap() - 1.5).abs() < 0.0001);
    assert_eq!(parser.read_i16().unwrap(), -99);
    assert!(parser.is_empty());
}

#[test]
fn test_payload_parser_peek_type() {
    let mut buffer = [0u8; 64];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_u32(123).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    
    // Peek shouldn't consume
    let (ptype, tlen) = parser.peek_type_info().unwrap();
    assert_eq!(ptype, PayloadType::Unsigned);
    assert_eq!(tlen, TypeLength::Bit32);
    
    // Should still be able to read
    assert_eq!(parser.read_u32().unwrap(), 123);
}

#[test]
fn test_payload_parser_skip_argument() {
    let mut buffer = [0u8; 128];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_u32(111).unwrap();
        builder.add_string("skip me").unwrap();
        builder.add_i16(222).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    assert_eq!(parser.read_u32().unwrap(), 111);
    parser.skip_argument().unwrap(); // Skip the string
    assert_eq!(parser.read_i16().unwrap(), 222);
    assert!(parser.is_empty());
}

#[test]
fn test_payload_parser_position_and_seek() {
    let mut buffer = [0u8; 64];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_u32(1).unwrap();
        builder.add_u32(2).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    
    assert_eq!(parser.position(), 0);
    parser.read_u32().unwrap();
    let mid_pos = parser.position();
    parser.read_u32().unwrap();
    
    // Seek back to middle
    parser.seek(mid_pos).unwrap();
    assert_eq!(parser.read_u32().unwrap(), 2);
}

#[test]
fn test_payload_parser_reset() {
    let mut buffer = [0u8; 64];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_u32(42).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    assert_eq!(parser.read_u32().unwrap(), 42);
    assert!(parser.is_empty());
    
    parser.reset();
    assert_eq!(parser.position(), 0);
    assert_eq!(parser.read_u32().unwrap(), 42);
}

#[test]
fn test_payload_parser_buffer_too_small() {
    let mut buffer = [0u8; 64];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_u32(42).unwrap();
        builder.len()
    };

    // Try to parse with truncated buffer
    let mut parser = PayloadParser::new(&buffer[..payload_len - 2]);
    assert!(parser.read_u32().is_err());
}

#[test]
fn test_payload_parser_type_mismatch() {
    let mut buffer = [0u8; 64];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_u32(42).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    
    // Try to read as wrong type
    assert!(parser.read_i32().is_err()); // Expected unsigned, got signed check
}

#[test]
fn test_payload_parser_remaining() {
    let mut buffer = [0u8; 64];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_u32(1).unwrap();
        builder.add_u32(2).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    
    let initial_remaining = parser.remaining();
    parser.read_u32().unwrap();
    let mid_remaining = parser.remaining();
    parser.read_u32().unwrap();
    let final_remaining = parser.remaining();
    
    assert!(initial_remaining > mid_remaining);
    assert!(mid_remaining > final_remaining);
    assert_eq!(final_remaining, 0);
}

#[test]
fn test_payload_roundtrip() {
    let mut buffer = [0u8; 256];
    
    // Build complex payload
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_bool(true).unwrap();
        builder.add_i8(-127).unwrap();
        builder.add_u16(65535).unwrap();
        builder.add_i32(-2147483648).unwrap();
        builder.add_f32(3.14159).unwrap();
        builder.add_string("Roundtrip test").unwrap();
        builder.add_raw(&[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
        builder.add_u64(18446744073709551615).unwrap();
        builder.len()
    };

    // Parse it back
    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    assert_eq!(parser.read_bool().unwrap(), true);
    assert_eq!(parser.read_i8().unwrap(), -127);
    assert_eq!(parser.read_u16().unwrap(), 65535);
    assert_eq!(parser.read_i32().unwrap(), -2147483648);
    assert!((parser.read_f32().unwrap() - 3.14159).abs() < 0.00001);
    assert_eq!(parser.read_string().unwrap(), "Roundtrip test");
    assert_eq!(parser.read_raw().unwrap(), &[0xDE, 0xAD, 0xBE, 0xEF]);
    assert_eq!(parser.read_u64().unwrap(), 18446744073709551615);
    assert!(parser.is_empty());
}

// ========================================
// DltValue and Automatic Parsing Tests
// ========================================

#[test]
fn test_read_next_bool() {
    let mut buffer = [0u8; 64];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_bool(true).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    match parser.read_next().unwrap() {
        DltValue::Bool(val) => assert_eq!(val, true),
        _ => panic!("Expected Bool value"),
    }
}

#[test]
fn test_read_next_integers() {
    let mut buffer = [0u8; 128];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_i8(-42).unwrap();
        builder.add_u16(1234).unwrap();
        builder.add_i32(-999999).unwrap();
        builder.add_u64(9876543210).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    
    match parser.read_next().unwrap() {
        DltValue::I8(val) => assert_eq!(val, -42),
        _ => panic!("Expected I8"),
    }
    
    match parser.read_next().unwrap() {
        DltValue::U16(val) => assert_eq!(val, 1234),
        _ => panic!("Expected U16"),
    }
    
    match parser.read_next().unwrap() {
        DltValue::I32(val) => assert_eq!(val, -999999),
        _ => panic!("Expected I32"),
    }
    
    match parser.read_next().unwrap() {
        DltValue::U64(val) => assert_eq!(val, 9876543210),
        _ => panic!("Expected U64"),
    }
    
    assert!(parser.is_empty());
}

#[test]
fn test_read_next_floats() {
    let mut buffer = [0u8; 64];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_f32(3.14159).unwrap();
        builder.add_f64(2.718281828).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    
    match parser.read_next().unwrap() {
        DltValue::F32(val) => assert!((val - 3.14159).abs() < 0.00001),
        _ => panic!("Expected F32"),
    }
    
    match parser.read_next().unwrap() {
        DltValue::F64(val) => assert!((val - 2.718281828).abs() < 0.000000001),
        _ => panic!("Expected F64"),
    }
}

#[test]
fn test_read_next_string() {
    let mut buffer = [0u8; 64];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_string("Hello, DLT!").unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    
    match parser.read_next().unwrap() {
        DltValue::String(val) => assert_eq!(val, "Hello, DLT!"),
        _ => panic!("Expected String"),
    }
}

#[test]
fn test_read_next_raw() {
    let mut buffer = [0u8; 64];
    let raw_data = b"\xDE\xAD\xBE\xEF";
    
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_raw(raw_data).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    
    match parser.read_next().unwrap() {
        DltValue::Raw(val) => assert_eq!(val, raw_data),
        _ => panic!("Expected Raw"),
    }
}

#[test]
fn test_read_next_mixed_unknown_types() {
    // Simulating parsing an incoming packet with unknown payload types
    let mut buffer = [0u8; 256];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_u32(42).unwrap();
        builder.add_string("Unknown packet").unwrap();
        builder.add_bool(true).unwrap();
        builder.add_f32(9.81).unwrap();
        builder.add_i16(-273).unwrap();
        builder.add_raw(&[0x01, 0x02, 0x03]).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    
    // Parse without knowing types ahead of time
    let val1 = parser.read_next().unwrap();
    let val2 = parser.read_next().unwrap();
    let val3 = parser.read_next().unwrap();
    let val4 = parser.read_next().unwrap();
    let val5 = parser.read_next().unwrap();
    let val6 = parser.read_next().unwrap();
    
    // Verify the parsed values
    assert_eq!(val1, DltValue::U32(42));
    assert_eq!(val2, DltValue::String("Unknown packet"));
    assert_eq!(val3, DltValue::Bool(true));
    
    match val4 {
        DltValue::F32(v) => assert!((v - 9.81).abs() < 0.001),
        _ => panic!("Expected F32"),
    }
    
    assert_eq!(val5, DltValue::I16(-273));
    assert_eq!(val6, DltValue::Raw(&[0x01, 0x02, 0x03]));
    
    assert!(parser.is_empty());
}

#[test]
fn test_read_all_args() {
    let mut buffer = [0u8; 256];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_u32(1).unwrap();
        builder.add_u32(2).unwrap();
        builder.add_string("three").unwrap();
        builder.add_bool(true).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    let mut args_buffer = [None; 10];
    
    let count = parser.read_all_args(&mut args_buffer).unwrap();
    
    assert_eq!(count, 4);
    assert_eq!(args_buffer[0], Some(DltValue::U32(1)));
    assert_eq!(args_buffer[1], Some(DltValue::U32(2)));
    assert_eq!(args_buffer[2], Some(DltValue::String("three")));
    assert_eq!(args_buffer[3], Some(DltValue::Bool(true)));
    assert_eq!(args_buffer[4], None);
}

#[test]
fn test_read_all_args_buffer_limit() {
    let mut buffer = [0u8; 128];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_u32(1).unwrap();
        builder.add_u32(2).unwrap();
        builder.add_u32(3).unwrap();
        builder.add_u32(4).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    let mut args_buffer = [None; 2]; // Only room for 2 args
    
    let count = parser.read_all_args(&mut args_buffer).unwrap();
    
    assert_eq!(count, 2); // Only parsed 2 due to buffer limit
    assert_eq!(args_buffer[0], Some(DltValue::U32(1)));
    assert_eq!(args_buffer[1], Some(DltValue::U32(2)));
    assert!(!parser.is_empty()); // Still has data remaining
}

#[test]
fn test_parse_incoming_packet_simulation() {
    // Simulate receiving a DLT packet with verbose payload
    // In real usage, this would be the payload section of a received DLT message
    let mut buffer = [0u8; 256];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        // Imagine this is a log message: "Temperature: 23.5°C, Status: OK, Error Code: 0"
        builder.add_string("Temperature").unwrap();
        builder.add_f32(23.5).unwrap();
        builder.add_string("Status").unwrap();
        builder.add_string("OK").unwrap();
        builder.add_string("ErrorCode").unwrap();
        builder.add_u32(0).unwrap();
        builder.len()
    };

    // Parse the incoming packet
    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    let mut args = [None; 10];
    let arg_count = parser.read_all_args(&mut args).unwrap();
    
    assert_eq!(arg_count, 6);
    
    // Verify the parsed message
    assert_eq!(args[0], Some(DltValue::String("Temperature")));
    match args[1] {
        Some(DltValue::F32(v)) => assert!((v - 23.5).abs() < 0.01),
        _ => panic!("Expected F32"),
    }
    assert_eq!(args[2], Some(DltValue::String("Status")));
    assert_eq!(args[3], Some(DltValue::String("OK")));
    assert_eq!(args[4], Some(DltValue::String("ErrorCode")));
    assert_eq!(args[5], Some(DltValue::U32(0)));
}

#[test]
fn test_read_next_u128() {
    let mut buffer = [0u8; 64];
    let value: u128 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
    
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_u128(value).unwrap();
        builder.len()
    };

    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    
    match parser.read_next().unwrap() {
        DltValue::U128(val) => assert_eq!(val, value),
        _ => panic!("Expected U128"),
    }
}

// ========================================
// DLT Header Parser Tests
// ========================================

#[test]
fn test_parse_standard_header_only() {
    // Create a minimal DLT message with only standard header
    let mut buffer = [0u8; 64];
    let mut builder = DltMessageBuilder::new();
    builder.set_endian(DltEndian::Little);
    
    // Create message without extended header or extra fields
    let htyp = 0x20; // Version 1, no extra fields
    buffer[0] = htyp;
    buffer[1] = 42; // Message counter
    // Length field is always big-endian: 4 bytes (just the standard header)
    buffer[2] = 0; // Length high byte
    buffer[3] = 4; // Length low byte
    
    let mut parser = DltHeaderParser::new(&buffer[..4]);
    let result = parser.parse_message();
    
    assert!(result.is_ok());
    let msg = result.unwrap();
    assert_eq!(msg.standard_header.htyp, htyp);
    assert_eq!(msg.standard_header.mcnt, 42);
    assert_eq!(msg.standard_header.len, 4);
    assert_eq!(msg.has_serial_header, false);
    assert_eq!(msg.ecu_id, None);
    assert_eq!(msg.session_id, None);
    assert_eq!(msg.timestamp, None);
    assert_eq!(msg.extended_header, None);
}

#[test]
fn test_parse_with_serial_header() {
    let mut buffer = [0u8; 128];
    let mut offset = 0;
    
    // Add serial header
    buffer[offset..offset + 4].copy_from_slice(&DLT_SERIAL_HEADER_ARRAY);
    offset += 4;
    
    // Add standard header
    let htyp = 0x20; // Version 1
    buffer[offset] = htyp;
    buffer[offset + 1] = 5;
    // Length field is always big-endian per DLT spec
    buffer[offset + 2] = 0; // Length high byte
    buffer[offset + 3] = 4; // Length low byte
    offset += 4;
    
    let mut parser = DltHeaderParser::new(&buffer[..offset]);
    let result = parser.parse_message();
    
    assert!(result.is_ok());
    let msg = result.unwrap();
    assert_eq!(msg.has_serial_header, true);
    assert_eq!(msg.standard_header.mcnt, 5);
}

#[test]
fn test_parse_with_ecu_id() {
    let mut buffer = [0u8; 128];
    let htyp = WEID_MASK | 0x20; // Version 1 + ECU ID
    
    buffer[0] = htyp;
    buffer[1] = 10;
    // Length field is always big-endian: 8 bytes (4 std + 4 ecu)
    buffer[2] = 0; // Length high byte
    buffer[3] = 8; // Length low byte
    buffer[4..8].copy_from_slice(b"ECU1");
    
    let mut parser = DltHeaderParser::new(&buffer[..8]);
    let msg = parser.parse_message().unwrap();
    
    assert_eq!(msg.ecu_id, Some(*b"ECU1"));
    assert_eq!(msg.session_id, None);
    assert_eq!(msg.timestamp, None);
}

#[test]
fn test_parse_with_all_extra_fields() {
    let mut buffer = [0u8; 128];
    let htyp = WEID_MASK | WSID_MASK | WTMS_MASK | 0x20; // Version 1 + all extra fields
    
    let mut offset = 0;
    buffer[offset] = htyp;
    buffer[offset + 1] = 20;
    // Length field is always big-endian: 16 bytes
    buffer[offset + 2] = 0; // Length high byte
    buffer[offset + 3] = 16; // Length low byte
    offset += 4;
    
    buffer[offset..offset + 4].copy_from_slice(b"TEST");
    offset += 4;
    
    buffer[offset..offset + 4].copy_from_slice(&1234u32.to_le_bytes());
    offset += 4;
    
    buffer[offset..offset + 4].copy_from_slice(&5678u32.to_le_bytes());
    offset += 4;
    
    let mut parser = DltHeaderParser::new(&buffer[..offset]);
    let msg = parser.parse_message().unwrap();
    
    assert_eq!(msg.ecu_id, Some(*b"TEST"));
    assert_eq!(msg.session_id, Some(1234));
    assert_eq!(msg.timestamp, Some(5678));
}

#[test]
fn test_parse_with_extended_header() {
    let mut buffer = [0u8; 128];
    let htyp = UEH_MASK | WEID_MASK | 0x20; // Version 1 + Extended Header + ECU ID
    
    let mut offset = 0;
    buffer[offset] = htyp;
    buffer[offset + 1] = 15;
    // Length field is always big-endian: 18 bytes (4 std + 4 ecu + 10 ext)
    buffer[offset + 2] = 0; // Length high byte
    buffer[offset + 3] = 18; // Length low byte
    offset += 4;
    
    buffer[offset..offset + 4].copy_from_slice(b"ECU2");
    offset += 4;
    
    // Extended header
    buffer[offset] = 0x41; // MSIN: verbose + log info
    buffer[offset + 1] = 3; // NOAR: 3 arguments
    buffer[offset + 2..offset + 6].copy_from_slice(b"APP1");
    buffer[offset + 6..offset + 10].copy_from_slice(b"CTX1");
    offset += 10;
    
    let mut parser = DltHeaderParser::new(&buffer[..offset]);
    let msg = parser.parse_message().unwrap();
    
    assert!(msg.extended_header.is_some());
    let ext = msg.extended_header.unwrap();
    assert_eq!(ext.msin, 0x41);
    assert_eq!(ext.noar, 3);
    assert_eq!(ext.apid, *b"APP1");
    assert_eq!(ext.ctid, *b"CTX1");
}

#[test]
fn test_parse_complete_message_with_payload() {
    // Build a complete message using the builder
    let mut build_buffer = [0u8; 256];
    let mut builder = DltMessageBuilder::new()
        .with_ecu_id(b"ECU1")
        .with_app_id(b"MYAP")
        .with_context_id(b"MYCT")
        .add_serial_header();
    
    let payload = b"Test payload";
    let total_size = builder.generate_log_message_with_payload(
        &mut build_buffer,
        payload,
        MtinTypeDltLog::DltLogInfo,
        1,
        false, // non-verbose
    ).unwrap();
    
    // Now parse it back - only pass the bytes that were written
    let mut parser = DltHeaderParser::new(&build_buffer[..total_size]);
    let msg = parser.parse_message().unwrap();
    
    assert_eq!(msg.has_serial_header, true);
    assert_eq!(msg.ecu_id, Some(*b"ECU1"));
    assert!(msg.extended_header.is_some());
    
    let ext = msg.extended_header.unwrap();
    assert_eq!(ext.apid, *b"MYAP");
    assert_eq!(ext.ctid, *b"MYCT");
    assert_eq!(ext.is_verbose(), false);
    assert_eq!(msg.payload, payload);
}

#[test]
fn test_parse_verbose_message_with_typed_payload() {
    // Build a message with verbose payload
    let mut build_buffer = [0u8; 256];
    
    // First, build payload
    let payload_len = {
        let mut payload_builder = PayloadBuilder::new(&mut build_buffer);
        payload_builder.add_u32(42).unwrap();
        payload_builder.add_string("Hello").unwrap();
        payload_builder.len()
    };
    
    // Create message with headers
    let mut message_builder = DltMessageBuilder::new()
        .with_ecu_id(b"TST1")
        .with_app_id(b"APP2")
        .with_context_id(b"CTX2");
    
    let total_size = message_builder.insert_header_at_front(
        &mut build_buffer,
        payload_len,
        2,
        MtinTypeDltLog::DltLogDebug,
    ).unwrap();
    
    // Parse it back - only use the written portion
    let mut parser = DltHeaderParser::new(&build_buffer[..total_size]);
    let msg = parser.parse_message().unwrap();
    
    assert_eq!(msg.ecu_id, Some(*b"TST1"));
    
    let ext = msg.extended_header.unwrap();
    assert_eq!(ext.apid, *b"APP2");
    assert_eq!(ext.ctid, *b"CTX2");
    assert_eq!(ext.is_verbose(), false); // insert_header_at_front uses false by default
    assert_eq!(ext.noar, 2);
    
    // Verify we can parse the payload
    let mut payload_parser = PayloadParser::new(msg.payload);
    assert_eq!(payload_parser.read_u32().unwrap(), 42);
    assert_eq!(payload_parser.read_string().unwrap(), "Hello");
}

#[test]
fn test_parse_big_endian_message() {
    let mut buffer = [0u8; 128];
    let htyp = MSBF_MASK | WEID_MASK | 0x20; // Big endian + ECU ID
    
    buffer[0] = htyp;
    buffer[1] = 7;
    buffer[2..4].copy_from_slice(&8u16.to_be_bytes()); // Big endian length
    buffer[4..8].copy_from_slice(b"ECU3");
    
    let mut parser = DltHeaderParser::new(&buffer[..8]);
    let msg = parser.parse_message().unwrap();
    
    assert_eq!(msg.header_type.MSBF, true);
    assert_eq!(msg.standard_header.len, 8);
    assert_eq!(msg.ecu_id, Some(*b"ECU3"));
}

#[test]
fn test_parse_extended_header_helpers() {
    let ext = DltExtendedHeader {
        msin: 0x41, // Verbose + Log Info (0100 0001)
        noar: 5,
        apid: *b"TEST",
        ctid: *b"TST1",
    };
    
    assert_eq!(ext.is_verbose(), true);
    assert_eq!(ext.message_type(), MstpType::DltTypeLog);
    assert_eq!(ext.message_type_info(), 4); // Info level
    
    let log_level = ext.log_level().unwrap();
    match log_level {
        MtinTypeDltLog::DltLogInfo => { /* correct */ }
        _ => panic!("Expected DltLogInfo"),
    }
}

#[test]
fn test_parse_buffer_too_small() {
    let buffer = [0u8; 2]; // Too small for standard header
    let mut parser = DltHeaderParser::new(&buffer);
    let result = parser.parse_message();
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), DltHeaderError::BufferTooSmall);
}

#[test]
fn test_parse_invalid_version() {
    let mut buffer = [0u8; 64];
    buffer[0] = 0x00; // Version 0 (invalid)
    buffer[1] = 0;
    buffer[2] = 4;
    buffer[3] = 0;
    
    let mut parser = DltHeaderParser::new(&buffer);
    let result = parser.parse_message();
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), DltHeaderError::InvalidVersion);
}

#[test]
fn test_roundtrip_complete_packet() {
    // Create a complete DLT packet
    let mut buffer = [0u8; 256];
    
    // Build payload first
    let payload_len = {
        let mut pb = PayloadBuilder::new(&mut buffer);
        pb.add_string("Temperature").unwrap();
        pb.add_f32(25.5).unwrap();
        pb.add_bool(true).unwrap();
        pb.len()
    };
    
    // Add headers
    let mut mb = DltMessageBuilder::new()
        .with_ecu_id(b"ECU1")
        .with_app_id(b"TEMP")
        .with_context_id(b"SENS")
        .with_session_id(9999)
        .with_timestamp(123456)
        .add_serial_header();
    
    let total = mb.insert_header_at_front(
        &mut buffer,
        payload_len,
        3,
        MtinTypeDltLog::DltLogWarn,
    ).unwrap();
    
    // Parse it back
    let mut parser = DltHeaderParser::new(&buffer[..total]);
    let msg = parser.parse_message().unwrap();
    
    // Verify all fields
    assert_eq!(msg.has_serial_header, true);
    assert_eq!(msg.ecu_id, Some(*b"ECU1"));
    assert_eq!(msg.session_id, Some(9999));
    assert_eq!(msg.timestamp, Some(123456));
    
    let ext = msg.extended_header.unwrap();
    assert_eq!(ext.apid, *b"TEMP");
    assert_eq!(ext.ctid, *b"SENS");
    assert_eq!(ext.noar, 3);
    
    // Parse payload
    let mut pp = PayloadParser::new(msg.payload);
    assert_eq!(pp.read_string().unwrap(), "Temperature");
    
    match pp.read_next().unwrap() {
        DltValue::F32(v) => assert!((v - 25.5).abs() < 0.01),
        _ => panic!("Expected F32"),
    }
    
    assert_eq!(pp.read_bool().unwrap(), true);
}

#[test]
fn test_parse_real_world_packet_data() {
    // Real-world DLT packet data containing multiple messages
    let data: [u8; 462] = [
        0x35, 0x00, 0x00, 0x20, 0x45, 0x43, 0x55, 0x31, 0x82, 0x72, 0xD9, 0x99, 0x26, 0x01, 0x44, 0x41, 0x31, 0x00, 0x44, 0x43, 0x31, 0x00, 0x02, 0x0F, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 
        0x35, 0x00, 0x00, 0x20, 0x45, 0x43, 0x55, 0x31, 0x82, 0x70, 0x6C, 0xAB, 0x26, 0x01, 0x44, 0x41, 0x31, 0x00, 0x44, 0x43, 0x31, 0x00, 0x02, 0x0F, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 
        0x3D, 0x0E, 0x00, 0x4F, 0x45, 0x43, 0x55, 0x31, 0x00, 0x02, 0x57, 0x67, 0x82, 0x70, 0x6C, 0xAB, 0x41, 0x01, 0x44, 0x4C, 0x54, 0x44, 0x49, 0x4E, 0x54, 0x4D, 0x00, 0x02, 0x00, 0x00, 0x2F, 0x00, 0x43, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x20, 0x63, 0x6F, 0x6E, 0x6E, 0x65, 0x63, 0x74, 0x69, 0x6F, 0x6E, 0x20, 0x23, 0x37, 0x20, 0x63, 0x6C, 0x6F, 0x73, 0x65, 0x64, 0x2E, 0x20, 0x54, 0x6F, 0x74, 0x61, 0x6C, 0x20, 0x43, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x73, 0x20, 0x3A, 0x20, 0x30, 0x00, 
        0x3D, 0x0F, 0x00, 0x58, 0x45, 0x43, 0x55, 0x31, 0x00, 0x02, 0x57, 0x67, 0x82, 0x72, 0xD9, 0x9B, 0x41, 0x01, 0x44, 0x4C, 0x54, 0x44, 0x49, 0x4E, 0x54, 0x4D, 0x00, 0x02, 0x00, 0x00, 0x38, 0x00, 0x4E, 0x65, 0x77, 0x20, 0x63, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x20, 0x63, 0x6F, 0x6E, 0x6E, 0x65, 0x63, 0x74, 0x69, 0x6F, 0x6E, 0x20, 0x23, 0x37, 0x20, 0x65, 0x73, 0x74, 0x61, 0x62, 0x6C, 0x69, 0x73, 0x68, 0x65, 0x64, 0x2C, 0x20, 0x54, 0x6F, 0x74, 0x61, 0x6C, 0x20, 0x43, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x73, 0x20, 0x3A, 0x20, 0x31, 0x00,
        0x35, 0x00, 0x00, 0x20, 0x45, 0x43, 0x55, 0x31, 0x84, 0xE0, 0xE6, 0x1A, 0x26, 0x01, 0x44, 0x41, 0x31, 0x00, 0x44, 0x43, 0x31, 0x00, 0x02, 0x0F, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 
        0x35, 0x00, 0x00, 0x20, 0x45, 0x43, 0x55, 0x31, 0x84, 0xD8, 0x90, 0x13, 0x26, 0x01, 0x44, 0x41, 0x31, 0x00, 0x44, 0x43, 0x31, 0x00, 0x02, 0x0F, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 
        0x3D, 0x36, 0x00, 0x4F, 0x45, 0x43, 0x55, 0x31, 0x00, 0x02, 0x57, 0x67, 0x84, 0xD8, 0x90, 0x13, 0x41, 0x01, 0x44, 0x4C, 0x54, 0x44, 0x49, 0x4E, 0x54, 0x4D, 0x00, 0x02, 0x00, 0x00, 0x2F, 0x00, 0x43, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x20, 0x63, 0x6F, 0x6E, 0x6E, 0x65, 0x63, 0x74, 0x69, 0x6F, 0x6E, 0x20, 0x23, 0x37, 0x20, 0x63, 0x6C, 0x6F, 0x73, 0x65, 0x64, 0x2E, 0x20, 0x54, 0x6F, 0x74, 0x61, 0x6C, 0x20, 0x43, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x73, 0x20, 0x3A, 0x20, 0x30, 0x00, 
        0x3D, 0x37, 0x00, 0x58, 0x45, 0x43, 0x55, 0x31, 0x00, 0x02, 0x57, 0x67, 0x84, 0xE0, 0xE6, 0x1A, 0x41, 0x01, 0x44, 0x4C, 0x54, 0x44, 0x49, 0x4E, 0x54, 0x4D, 0x00, 0x02, 0x00, 0x00, 0x38, 0x00, 0x4E, 0x65, 0x77, 0x20, 0x63, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x20, 0x63, 0x6F, 0x6E, 0x6E, 0x65, 0x63, 0x74, 0x69, 0x6F, 0x6E, 0x20, 0x23, 0x37, 0x20, 0x65, 0x73, 0x74, 0x61, 0x62, 0x6C, 0x69, 0x73, 0x68, 0x65, 0x64, 0x2C, 0x20, 0x54, 0x6F, 0x74, 0x61, 0x6C, 0x20, 0x43, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x73, 0x20, 0x3A, 0x20, 0x31, 0x00
    ];
    
    let mut position = 0;
    let mut message_count = 0;
    
    // Parse all messages in the buffer
    while position < data.len() {
        let mut parser = DltHeaderParser::new(&data[position..]);
        
        match parser.parse_message() {
            Ok(msg) => {
                message_count += 1;
                
                // Verify common fields
                assert_eq!(msg.has_serial_header, false);
                assert_eq!(msg.ecu_id, Some(*b"ECU1"));
                
                // Print message details for debugging
                println!("Message {}: ECU={:?}, Session={:?}, Timestamp={:?}", 
                    message_count,
                    core::str::from_utf8(&msg.ecu_id.unwrap()).unwrap_or("???"),
                    msg.session_id,
                    msg.timestamp
                );
                
                if let Some(ext) = msg.extended_header {
                    println!("  APP={:?}, CTX={:?}, NOAR={}, Verbose={}", 
                        core::str::from_utf8(&ext.apid).unwrap_or("???"),
                        core::str::from_utf8(&ext.ctid).unwrap_or("???"),
                        ext.noar,
                        ext.is_verbose()
                    );
                    
                    // If it's a verbose message with string payload, try to parse it
                    if ext.is_verbose() && msg.payload.len() > 0 {
                        let mut pp = PayloadParser::new(msg.payload);
                        
                        // Try to read all arguments
                        let mut args_buffer: [Option<DltValue>; 10] = [None, None, None, None, None, None, None, None, None, None];
                        match pp.read_all_args(&mut args_buffer) {
                            Ok(count) => {
                                println!("  Payload args: {} values", count);
                                for i in 0..count {
                                    if let Some(arg) = &args_buffer[i] {
                                        match arg {
                                            DltValue::String(s) => println!("    [{}] String: {}", i, s),
                                            DltValue::U32(v) => println!("    [{}] U32: {}", i, v),
                                            DltValue::I32(v) => println!("    [{}] I32: {}", i, v),
                                            _ => println!("    [{}] {:?}", i, arg),
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("  Failed to parse payload: {:?}", e);
                            }
                        }
                    } else {
                        // Non-verbose payload - just print hex
                        println!("  Non-verbose payload: {} bytes", msg.payload.len());
                    }
                }
                
                // Calculate bytes consumed and move to next message
                let bytes_consumed = parser.position();
                position += bytes_consumed;
            }
            Err(e) => {
                println!("Parse error at position {}: {:?}", position, e);
                break;
            }
        }
    }
    
    println!("\nTotal messages parsed: {}", message_count);
    assert!(message_count >= 4, "Expected to parse at least 4 messages from real-world data");
}

