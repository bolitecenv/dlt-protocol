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

pub struct DltMessageBuilder<'a> {
    header_htyp: u8,
    message_counter: u8,
    serial_header: bool,
    ecu_id: &'a [u8; DLT_ID_SIZE],
    pub session_id: u32,
    pub timestamp: u32,
    app_id: &'a [u8; DLT_ID_SIZE],
    context_id: &'a [u8; DLT_ID_SIZE],
    endian: DltEndian,
    get_tmsp: Option<fn() -> u32>,
    get_sess_id: Option<fn() -> u32>,
    timestamp_provider: Option<&'static dyn TimestampProvider>,
    session_id_provider: Option<&'static dyn SessionIdProvider>,
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
            serial_header: false,
            timestamp_provider: GLOBAL_TIMESTAMP.get(),
            session_id_provider: GLOBAL_SESSION.get(),
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

    pub fn with_session_id(mut self, sess_id: u32) -> Self {
        self.session_id = sess_id;
        self
    }

    pub fn with_timestamp(mut self, timestamp: u32) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn add_serial_header(mut self) -> Self {
        self.serial_header = true;
        self
    }

    pub fn set_timestamp_provider(&mut self, provider: &'static dyn TimestampProvider) {
        self.timestamp_provider = Some(provider);
    }

    pub fn set_session_id_provider(&mut self, provider: &'static dyn SessionIdProvider) {
        self.session_id_provider = Some(provider);
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

    pub fn insert_header_at_front(&mut self, buffer: &mut [u8], payload_size: usize, arg_num: u8, log_level: MtinTypeDltLog) -> Result<usize, DltError> {
        let header_size = self._generate_log_message_header_size();
        let serial_size = if self.serial_header { DLT_SERIAL_HEADER_SIZE } else { 0 };
        let total_header_size = serial_size + header_size;
        
        if buffer.len() < total_header_size {
            return Err(DltError::BufferTooSmall);
        }

        let total_size = total_header_size + payload_size;
        if buffer.len() < total_size {
            return Err(DltError::BufferTooSmall);
        }

        // Move existing payload data to make space for header (serial + standard + extra + extended)
        for i in (0..payload_size).rev() {
            buffer[i + total_header_size] = buffer[i];
        }

        // Insert header at the front of the buffer
        self._generate_log_message(buffer, payload_size, log_level, arg_num, false)?;
        
        Ok(total_size)
    }

    pub fn generate_log_message_with_payload(
        &mut self,
        buffer: &mut [u8],
        payload: &[u8],
        log_level: MtinTypeDltLog,
        number_of_arguments: u8,
        verbose: bool,
    ) -> Result<usize, DltError> {
        let payload_size = payload.len();
        let header_size = self._generate_log_message_header_size();
        let serial_size = if self.serial_header { DLT_SERIAL_HEADER_SIZE } else { 0 };
        let total_size = serial_size + header_size + payload_size;

        if buffer.len() < total_size {
            return Err(DltError::BufferTooSmall);
        }

        // Generate header
        let header_bytes_written = self._generate_log_message(buffer, payload_size, log_level, number_of_arguments, verbose)?;

        // Copy payload after header
        buffer[header_bytes_written..header_bytes_written + payload_size]
            .copy_from_slice(payload);

        Ok(total_size)
    }
    
    /// Generate a DLT log message
    /// Returns the number of bytes written to the buffer
    pub fn _generate_log_message(
        &mut self,
        buffer: &mut [u8],
        payload_size: usize,
        log_level: MtinTypeDltLog,
        number_of_arguments: u8,
        verbose: bool,
    ) -> Result<usize, DltError> {
        let mut offset = 0;
        let mut total_len;
        let len_field; // Length field value (excludes serial header)

        // Calculate total message length
        if self.serial_header {
            total_len = DLT_SERIAL_HEADER_SIZE;
        } else {
            total_len = 0;
        }
        total_len += self._generate_log_message_header_size();
        total_len += payload_size;
        
        // Length field excludes serial header
        len_field = (total_len - if self.serial_header { DLT_SERIAL_HEADER_SIZE } else { 0 }) as u16;

        if buffer.len() < total_len {
            return Err(DltError::BufferTooSmall);
        }

        // Write Serial Header if enabled
        if self.serial_header {
            buffer[offset..offset + DLT_SERIAL_HEADER_SIZE].copy_from_slice(&DLT_SERIAL_HEADER_ARRAY);
            offset += DLT_SERIAL_HEADER_SIZE;
        }

        // Write Standard Header
        // Update MSBF bit based on endianness
        let mut htyp = self.header_htyp;
        match self.endian {
            DltEndian::Big => htyp |= DltHeaderHtyp::MSBF_MASK,
            DltEndian::Little => htyp &= !DltHeaderHtyp::MSBF_MASK,
        }
        buffer[offset] = htyp;
        offset += 1;
        buffer[offset] = self.message_counter;
        offset += 1;

        // Write length (excludes serial header)
        // Per DLT spec PRS_Dlt_00091: Standard header uses network byte order (big-endian)
        buffer[offset..offset + 2].copy_from_slice(&len_field.to_be_bytes());
        offset += 2;

        // Write Standard Header Extra fields (conditional based on HTYP flags)
        
        // ECU ID (if WEID bit is set)
        if self.header_htyp & DltHeaderHtyp::WEID_MASK != 0 {
            buffer[offset..offset + DLT_ID_SIZE].copy_from_slice(self.ecu_id);
            offset += DLT_ID_SIZE;
        }

        // Session ID (if WSID bit is set)
        if self.header_htyp & DltHeaderHtyp::WSID_MASK != 0 {
            // If session ID getter is provided, use it
            if let Some(provider) = &self.session_id_provider {
                self.session_id = provider.get_session_id();
            }
            buffer[offset..offset + 4]
                .copy_from_slice(&switch_endian_u32(self.session_id, &self.endian));
            offset += 4;
        }

        // Timestamp (if WTMS bit is set)
        if self.header_htyp & DltHeaderHtyp::WTMS_MASK != 0 {
            // If timestamp getter is provided, use it
            if let Some(provider) = &self.timestamp_provider {
                self.timestamp = provider.get_timestamp();
            }
            buffer[offset..offset + 4]
                .copy_from_slice(&switch_endian_u32(self.timestamp, &self.endian));
            offset += 4;
        }

        // Write Extended Header
        let verbose_bit = if verbose { 0x01 } else { 0x00 };
        let mtin_bits = (log_level.to_bits() & 0x0F) << 4;
        let mstp_bits = (MstpType::DltTypeLog.to_bits() & 0x07) << 1;
        let msin = verbose_bit | mtin_bits | mstp_bits;

        buffer[offset] = msin;
        offset += 1;

        buffer[offset] = number_of_arguments;
        offset += 1;

        // APP ID
        buffer[offset..offset + DLT_ID_SIZE].copy_from_slice(self.app_id);
        offset += DLT_ID_SIZE;

        // Context ID
        buffer[offset..offset + DLT_ID_SIZE].copy_from_slice(self.context_id);
        offset += DLT_ID_SIZE;

        // Increment message counter for next message
        self.increment_counter();

        Ok(offset)
    }

    fn _standard_header_extra_size(&self) -> usize {
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

    fn _generate_log_message_header_size(&self) -> usize {
        let mut total_len = 0;

        // This function returns header size WITHOUT serial header
        total_len += DLT_STANDARD_HEADER_SIZE;
        total_len += self._standard_header_extra_size(); // ECU ID + Session ID + Timestamp
        total_len += DLT_EXTENDED_HEADER_SIZE;

        total_len
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
