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

pub struct DltMessageBuilder<'a> {
    header_htyp: u8,
    message_counter: u8,
    ecu_id: &'a [u8; DLT_ID_SIZE],
    session_id: u32,
    timestamp: u32,
    app_id: &'a [u8; DLT_ID_SIZE],
    context_id: &'a [u8; DLT_ID_SIZE],
    endian: DltEndian,
    get_tmsp: Option<fn() -> u32>,
    get_sess_id: Option<fn() -> u32>,
}

impl<'a> DltMessageBuilder<'a> {
    pub fn new() -> Self {
        // Define a static default ECU ID to ensure a reference with the correct type and lifetime
        static DEFAULT_ECU_ID: [u8; DLT_ID_SIZE] = *b"ECU\0";
        static DEFAULT_APP_ID: [u8; DLT_ID_SIZE] = *b"APP\0";
        static DEFAULT_CTX_ID: [u8; DLT_ID_SIZE] = *b"CTX\0";
        Self {
            header_htyp: UEH_MASK
                | DltHeaderHtyp::WEID_MASK
                | DltHeaderHtyp::WSID_MASK
                | DltHeaderHtyp::WTMS_MASK
                | (VERS_MASK & (0x1 << 5)), // Version 1
            message_counter: 0,
            ecu_id: &DEFAULT_ECU_ID,
            session_id: 0,
            timestamp: 0,
            app_id: &DEFAULT_APP_ID,
            context_id: &DEFAULT_CTX_ID,
            endian: DltEndian::Big,
            get_tmsp: None,
            get_sess_id: None,
        }
    }

    pub fn htyp(mut self, htyp: u8) -> Self {
        self.header_htyp = htyp;
        self
    }

    pub fn msg_counter(mut self, msg_cnt: u8) -> Self {
        self.message_counter = msg_cnt;
        self
    }

    pub fn with_ecu_id(mut self, ecu_id: &'a [u8; DLT_ID_SIZE]) -> Self {
        self.ecu_id = ecu_id;
        self
    }

    pub fn with_app_id(mut self, app_id: &'a [u8; DLT_ID_SIZE]) -> Self {
        self.app_id = app_id;
        self
    }

    pub fn with_context_id(mut self, ctx_id: &'a [u8; DLT_ID_SIZE]) -> Self {
        self.context_id = ctx_id;
        self
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

    pub fn set_timestamp_getter(&mut self, getter: fn() -> u32) {
        self.get_tmsp = Some(getter);
    }

    pub fn set_session_id_getter(&mut self, getter: fn() -> u32) {
        self.get_sess_id = Some(getter);
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
        buffer[offset..offset + DLT_ID_SIZE].copy_from_slice(self.ecu_id);
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
        buffer[offset..offset + DLT_ID_SIZE].copy_from_slice(self.app_id);
        offset += DLT_ID_SIZE;

        // Context ID
        buffer[offset..offset + DLT_ID_SIZE].copy_from_slice(self.context_id);
        offset += DLT_ID_SIZE;

        // Write Payload
        buffer[offset..offset + payload.len()].copy_from_slice(payload);
        offset += payload.len();

        // Increment message counter for next message
        self.increment_counter();

        Ok(offset)
    }

    #[inline]
    pub fn log_text(
        &mut self,
        buffer: &mut [u8],
        log_level: MtinTypeDltLog,
        text: &[u8],
    ) -> Result<usize, DltError> {
        self.generate_log_message(buffer, log_level, text, false)
    }

    #[inline]
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

// Helper function to convert binary to fixed size array for DLT ID
#[inline]
pub fn to_dlt_id_array(id: &[u8]) -> [u8; DLT_ID_SIZE] {
    let mut array = [0u8; DLT_ID_SIZE];
    let len = core::cmp::min(id.len(), DLT_ID_SIZE);
    array[..len].copy_from_slice(&id[..len]);
    array
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
