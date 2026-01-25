use crate::r19_11::*;

// DLT Message Builder
#[derive(Debug)]
pub struct DltMessageBuilder {
    message_counter: u8,
    ecu_id: [u8; DLT_ID_SIZE],
    session_id: u32,
    timestamp: u32,
    app_id: [u8; DLT_ID_SIZE],
    context_id: [u8; DLT_ID_SIZE],
}

impl DltMessageBuilder {
    pub fn new() -> Self {
        Self {
            message_counter: 0,
            ecu_id: *b"ECU1",
            session_id: 0,
            timestamp: 0,
            app_id: *b"APP1",
            context_id: *b"CTX1",
        }
    }

    pub fn with_ecu_id(mut self, ecu_id: [u8; DLT_ID_SIZE]) -> Self {
        self.ecu_id = ecu_id;
        self
    }

    pub fn with_app_id(mut self, app_id: [u8; DLT_ID_SIZE]) -> Self {
        self.app_id = app_id;
        self
    }

    pub fn with_context_id(mut self, context_id: [u8; DLT_ID_SIZE]) -> Self {
        self.context_id = context_id;
        self
    }

    pub fn with_session_id(mut self, session_id: u32) -> Self {
        self.session_id = session_id;
        self
    }

    pub fn set_timestamp(&mut self, timestamp: u32) {
        self.timestamp = timestamp;
    }

    pub fn increment_counter(&mut self) {
        self.message_counter = self.message_counter.wrapping_add(1);
    }

    /// Generate a DLT log message
    /// Returns the number of bytes written to the buffer
    pub fn generate_log_message(
        &mut self,
        buffer: &mut [u8],
        log_level: MtinTypeDltLog,
        payload: &[u8],
        verbose: bool,
    ) -> Result<usize, DltError> {
        let mut offset = 0;

        // Calculate total message length
        let mut total_len = DLT_STANDARD_HEADER_SIZE;
        total_len += DLT_STANDARD_HEADER_EXTRA_SIZE; // ECU ID + Session ID + Timestamp
        total_len += DLT_EXTENDED_HEADER_SIZE;
        total_len += payload.len();

        if buffer.len() < total_len {
            return Err(DltError::BufferTooSmall);
        }

        // Build HTYP byte
        let htyp = DLT_VERSION | UEH_MASK | WEID_MASK | WSID_MASK | WTMS_MASK;

        // Write Standard Header
        buffer[offset] = htyp;
        offset += 1;
        buffer[offset] = self.message_counter;
        offset += 1;

        // Write length (big-endian)
        let len = total_len as u16;
        buffer[offset..offset + 2].copy_from_slice(&len.to_be_bytes());
        offset += 2;

        // Write Standard Header Extra
        buffer[offset..offset + 4].copy_from_slice(&self.ecu_id);
        offset += 4;
        buffer[offset..offset + 4].copy_from_slice(&self.session_id.to_be_bytes());
        offset += 4;
        buffer[offset..offset + 4].copy_from_slice(&self.timestamp.to_be_bytes());
        offset += 4;

        // Write Extended Header
        // MSIN byte: Message Info
        // Bit 0: Verbose mode (1 = verbose, 0 = non-verbose)
        // Bit 1-3: Message Type Info (MTIN)
        // Bit 4-6: Message Type (MSTP)
        // Bit 7: UEH flag (always 0 in extended header)
        let verbose_bit = if verbose { 0x01 } else { 0x00 };
        let mtin_bits = (log_level.to_bits() & 0x0F) << 1;
        let mstp_bits = (MstpType::DltTypeLog.to_bits() & 0x07) << 4;
        let msin = verbose_bit | mtin_bits | mstp_bits;

        buffer[offset] = msin;
        offset += 1;

        // NOAR: Number of arguments (0 for simple string payload)
        buffer[offset] = if verbose { 1 } else { 0 };
        offset += 1;

        // APP ID
        buffer[offset..offset + 4].copy_from_slice(&self.app_id);
        offset += 4;

        // Context ID
        buffer[offset..offset + 4].copy_from_slice(&self.context_id);
        offset += 4;

        // Write Payload
        buffer[offset..offset + payload.len()].copy_from_slice(payload);
        offset += payload.len();

        // Increment message counter for next message
        self.increment_counter();

        Ok(offset)
    }

    /// Generate a simple text log message (non-verbose mode)
    pub fn log_text(
        &mut self,
        buffer: &mut [u8],
        log_level: MtinTypeDltLog,
        text: &[u8],
    ) -> Result<usize, DltError> {
        self.generate_log_message(buffer, log_level, text, false)
    }

    /// Generate a verbose log message with type info
    pub fn log_verbose(
        &mut self,
        buffer: &mut [u8],
        log_level: MtinTypeDltLog,
        payload: &[u8],
    ) -> Result<usize, DltError> {
        self.generate_log_message(buffer, log_level, payload, true)
    }
}

impl Default for DltMessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for creating common log messages
pub fn make_log_fatal(
    builder: &mut DltMessageBuilder,
    buffer: &mut [u8],
    text: &[u8],
) -> Result<usize, DltError> {
    builder.log_text(buffer, MtinTypeDltLog::DltLogFatal, text)
}

pub fn make_log_error(
    builder: &mut DltMessageBuilder,
    buffer: &mut [u8],
    text: &[u8],
) -> Result<usize, DltError> {
    builder.log_text(buffer, MtinTypeDltLog::DltLogError, text)
}

pub fn make_log_warn(
    builder: &mut DltMessageBuilder,
    buffer: &mut [u8],
    text: &[u8],
) -> Result<usize, DltError> {
    builder.log_text(buffer, MtinTypeDltLog::DltLogWarn, text)
}

pub fn make_log_info(
    builder: &mut DltMessageBuilder,
    buffer: &mut [u8],
    text: &[u8],
) -> Result<usize, DltError> {
    builder.log_text(buffer, MtinTypeDltLog::DltLogInfo, text)
}

pub fn make_log_debug(
    builder: &mut DltMessageBuilder,
    buffer: &mut [u8],
    text: &[u8],
) -> Result<usize, DltError> {
    builder.log_text(buffer, MtinTypeDltLog::DltLogDebug, text)
}

pub fn make_log_verbose(
    builder: &mut DltMessageBuilder,
    buffer: &mut [u8],
    text: &[u8],
) -> Result<usize, DltError> {
    builder.log_text(buffer, MtinTypeDltLog::DltLogVerbose, text)
}
