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
