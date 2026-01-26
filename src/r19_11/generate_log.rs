#![no_std]

use crate::r19_11::*;

struct DltHeaderHtyp;

impl DltHeaderHtyp {
    const UEH_MASK: u8 = 0x01;
    const MSBF_MASK: u8 = 0x02;
    const WEID_MASK: u8 = 0x04;
    const WSID_MASK: u8 = 0x08;
    const WTMS_MASK: u8 = 0x10;
    const VERS_MASK: u8 = 0xE0;
}

pub enum DltEndian {
    Big,
    Little,
}

pub struct DltCommonSettings {
    pub endian: DltEndian,
}

pub struct DltCommonGetHandler {
    pub get_timestamp: Option<fn() -> u32>,
    pub get_session_id: Option<fn() -> u32>,
}

pub struct DltCommonBuilder {
    pub settings: DltCommonSettings,
    pub handlers: DltCommonGetHandler,
}

// DLT Common Message Builder thread safe, no_std static instance
// Note: This requires unsafe or once_cell/lazy_static for true static initialization
// with non-const values. Using const-compatible default here.
pub static DLT_COMMON_BUILDER: DltCommonBuilder = DltCommonBuilder {
    settings: DltCommonSettings {
        endian: DltEndian::Big,
    },
    handlers: DltCommonGetHandler {
        get_timestamp: None,
        get_session_id: None,
    },
};

pub struct DltMessageBuilder {
    header_htyp: u8,
    message_counter: u8,
    ecu_id: [u8; DLT_ID_SIZE],
    session_id: u32,
    timestamp: u32,
    app_id: [u8; DLT_ID_SIZE],
    context_id: [u8; DLT_ID_SIZE],
    endian: DltEndian,
    get_tmsp: Option<fn() -> u32>,
    get_sess_id: Option<fn() -> u32>,
}

impl DltMessageBuilder {
    pub fn new(htyp: u8, msg_cnt: u8, ecu_id: &str, app_id: &str, ctx_id: &str) -> Self {
        // Helper function to convert string to fixed-size array
        fn str_to_id(s: &str, default: &[u8; DLT_ID_SIZE]) -> [u8; DLT_ID_SIZE] {
            let mut id = *default;
            let bytes = s.as_bytes();
            let len = bytes.len().min(DLT_ID_SIZE);
            id[..len].copy_from_slice(&bytes[..len]);
            id
        }

        Self {
            header_htyp: htyp,
            message_counter: msg_cnt,
            ecu_id: str_to_id(ecu_id, b"ECU1"),
            session_id: 0,
            timestamp: 0,
            app_id: str_to_id(app_id, b"APP1"),
            context_id: str_to_id(ctx_id, b"CTX1"),
            endian: DltEndian::Big,
            get_tmsp: None,
            get_sess_id: None,
        }
    }

    pub fn increment_counter(&mut self) {
        self.message_counter = self.message_counter.wrapping_add(1);
    }

    pub fn reset_counter(&mut self) {
        self.message_counter = 0;
    }

    pub fn get_counter(&self) -> u8 {
        self.message_counter
    }

    pub fn set_endian(&mut self, endian: DltEndian) {
        self.endian = endian;
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
        total_len += self.standard_header_extra_size(); // ECU ID + Session ID + Timestamp
        total_len += DLT_EXTENDED_HEADER_SIZE;
        total_len += payload.len();

        if buffer.len() < total_len {
            return Err(DltError::BufferTooSmall);
        }

        // Write Standard Header
        buffer[offset] = self.header_htyp;
        offset += 1;
        buffer[offset] = self.message_counter;
        offset += 1;

        // Write length
        let len = total_len as u16;
        buffer[offset..offset + 2].copy_from_slice(&switch_endian_u16(len, &self.endian));
        offset += 2;

        // Write Standard Header Extra
        buffer[offset..offset + DLT_ID_SIZE].copy_from_slice(&self.ecu_id);
        offset += DLT_ID_SIZE;

        // If session ID getter is provided, use it
        if let Some(get_sess_id) = self.get_sess_id {
            self.session_id = get_sess_id();
        }
        // If timestamp getter is provided, use it
        if let Some(get_tmsp) = self.get_tmsp {
            self.timestamp = get_tmsp();
        }

        buffer[offset..offset + 4]
            .copy_from_slice(&switch_endian_u32(self.session_id, &self.endian));
        offset += 4;
        buffer[offset..offset + 4]
            .copy_from_slice(&switch_endian_u32(self.timestamp, &self.endian));
        offset += 4;

        // Write Extended Header
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
        buffer[offset..offset + DLT_ID_SIZE].copy_from_slice(&self.app_id);
        offset += DLT_ID_SIZE;

        // Context ID
        buffer[offset..offset + DLT_ID_SIZE].copy_from_slice(&self.context_id);
        offset += DLT_ID_SIZE;

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

    fn standard_header_extra_size(&self) -> usize {
        let mut size = 0;
        if self.header_htyp & DltHeaderHtyp::WEID_MASK != 0 {
            size += DLT_ID_SIZE; // ECU ID
        }
        if self.header_htyp & DltHeaderHtyp::WSID_MASK != 0 {
            size += 4; // Session ID (not APP ID)
        }
        if self.header_htyp & DltHeaderHtyp::WTMS_MASK != 0 {
            size += 4; // Timestamp
        }
        size
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

// switch endian byte array depends the DltEndian enum
#[inline]
fn switch_endian_u32(value: u32, endian: &DltEndian) -> [u8; 4] {
    match endian {
        DltEndian::Big => value.to_be_bytes(),
        DltEndian::Little => value.to_le_bytes(),
    }
}

#[inline]
fn switch_endian_u16(value: u16, endian: &DltEndian) -> [u8; 2] {
    match endian {
        DltEndian::Big => value.to_be_bytes(),
        DltEndian::Little => value.to_le_bytes(),
    }
}
