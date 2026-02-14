//! # DLT Log Message Generator
//!
//! This module provides the builder pattern API for creating DLT protocol messages
//! according to the AUTOSAR DLT specification release 19.11.
//!
//! ## Usage
//!
//! ```no_run
//! use dlt_protocol::r19_11::*;
//!
//! let mut builder = DltMessageBuilder::new()
//!     .with_ecu_id(b"ECU1")
//!     .with_app_id(b"APP1")
//!     .with_context_id(b"CTX1")
//!     .add_serial_header();
//!
//! let mut buffer = [0u8; 256];
//! let payload = b"Hello, DLT!";
//! let size = builder.generate_log_message_with_payload(
//!     &mut buffer,
//!     payload,
//!     MtinTypeDltLog::DltLogInfo,
//!     1,
//!     true,
//! ).unwrap();
//! ```

use crate::r19_11::*;

// ========================================
// HTYP Bit Masks (duplicated for internal use)
// ========================================

struct DltHeaderHtyp;

impl DltHeaderHtyp {
    const UEH_MASK: u8 = 0x01;
    const MSBF_MASK: u8 = 0x02;
    const WEID_MASK: u8 = 0x04;
    const WSID_MASK: u8 = 0x08;
    const WTMS_MASK: u8 = 0x10;
    const VERS_MASK: u8 = 0xE0;
}

// ========================================
// Endianness Configuration
// ========================================

/// Byte order for multi-byte fields in DLT messages
pub enum DltEndian {
    /// Big-endian (network byte order)
    Big,
    /// Little-endian
    Little,
}

// ========================================
// Common Builder Configuration (Legacy) 
// ========================================

/// Common settings for DLT message generation
pub struct DltCommonSettings {
    pub endian: DltEndian,
}

/// Handler functions for dynamic values
pub struct DltCommonGetHandler {
    pub get_timestamp: Option<fn() -> u32>,
    pub get_session_id: Option<fn() -> u32>,
}

/// Common builder configuration (legacy)
pub struct DltCommonBuilder {
    pub settings: DltCommonSettings,
    pub handlers: DltCommonGetHandler,
}

// ========================================
// DLT Message Builder
// ========================================

/// Builder for creating DLT protocol messages
///
/// This struct provides a fluent API for configuring and generating DLT messages.
/// It maintains state for message counters, IDs, and timestamp/session providers.
///
/// # Example
/// ```no_run
/// use dlt_protocol::r19_11::*;
///
/// let mut builder = DltMessageBuilder::new()
///     .with_ecu_id(b"ECU1")
///     .with_app_id(b"MYAP")
///     .with_context_id(b"CTX1");
///
/// let mut buffer = [0u8; 256];
/// let size = builder.generate_log_message_with_payload(
///     &mut buffer,
///     b"Log message",
///     MtinTypeDltLog::DltLogInfo,
///     1,
///     true,
/// ).unwrap();
/// ```
pub struct DltMessageBuilder<'a> {
    /// Standard header HTYP byte (flags for optional fields)
    header_htyp: u8,
    /// Message counter (auto-incremented, wraps at 255)
    message_counter: u8,
    /// Whether to include serial header ("DLS\x01")
    serial_header: bool,
    /// ECU ID (4 bytes)
    ecu_id: &'a [u8; DLT_ID_SIZE],
    /// Session ID value
    pub session_id: u32,
    /// Timestamp value (in 0.1ms units)
    pub timestamp: u32,
    /// Application ID (4 bytes)
    app_id: &'a [u8; DLT_ID_SIZE],
    /// Context ID (4 bytes)
    context_id: &'a [u8; DLT_ID_SIZE],
    /// Byte order for multi-byte fields
    endian: DltEndian,
    /// Legacy timestamp getter function
    get_tmsp: Option<fn() -> u32>,
    /// Legacy session ID getter function
    get_sess_id: Option<fn() -> u32>,
    /// Global timestamp provider (preferred)
    timestamp_provider: Option<&'static dyn TimestampProvider>,
    /// Global session ID provider (preferred)
    session_id_provider: Option<&'static dyn SessionIdProvider>,
}

impl<'a> DltMessageBuilder<'a> {
    /// Create a new message builder with default settings
    ///
    /// Default configuration:
    /// - Extended header enabled (UEH)
    /// - ECU ID enabled (WEID)
    /// - Session ID enabled (WSID)
    /// - Timestamp enabled (WTMS)
    /// - Version 1.0
    /// - Big-endian
    /// - Default IDs: ECU\0, APP\0, CTX\0
    /// - Message counter starts at 0
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
            endian: DltEndian::Little,
            get_tmsp: None,
            get_sess_id: None,
            serial_header: false,
            timestamp_provider: GLOBAL_TIMESTAMP.get(),
            session_id_provider: GLOBAL_SESSION.get(),
        }
    }

    // ========================================
    // Configuration Methods (Builder Pattern)
    // ========================================

    /// Set custom HTYP byte directly (advanced usage)
    pub fn htyp(mut self, htyp: u8) -> Self {
        self.header_htyp = htyp;
        self
    }

    /// Set message counter to a specific value
    pub fn msg_counter(mut self, msg_cnt: u8) -> Self {
        self.message_counter = msg_cnt;
        self
    }

    /// Set ECU ID (4 bytes, e.g., b"ECU1")
    pub fn with_ecu_id(mut self, ecu_id: &'a [u8; DLT_ID_SIZE]) -> Self {
        self.ecu_id = ecu_id;
        self
    }

    /// Set Application ID (4 bytes, e.g., b"MYAP")
    pub fn with_app_id(mut self, app_id: &'a [u8; DLT_ID_SIZE]) -> Self {
        self.app_id = app_id;
        self
    }

    /// Set Context ID (4 bytes, e.g., b"CTX1")
    pub fn with_context_id(mut self, ctx_id: &'a [u8; DLT_ID_SIZE]) -> Self {
        self.context_id = ctx_id;
        self
    }

    /// Set static session ID value
    pub fn with_session_id(mut self, sess_id: u32) -> Self {
        self.session_id = sess_id;
        self
    }

    /// Set static timestamp value (in 0.1ms units)
    pub fn with_timestamp(mut self, timestamp: u32) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Enable serial header ("DLS\x01") at the beginning of messages
    pub fn add_serial_header(mut self) -> Self {
        self.serial_header = true;
        self
    }

    // ========================================
    // Dynamic Value Providers
    // ========================================

    /// Set a timestamp provider for dynamic timestamp values
    ///
    /// The provider will be called each time a message is generated.
    pub fn set_timestamp_provider(&mut self, provider: &'static dyn TimestampProvider) {
        self.timestamp_provider = Some(provider);
    }

    /// Set a session ID provider for dynamic session ID values
    ///
    /// The provider will be called each time a message is generated.
    pub fn set_session_id_provider(&mut self, provider: &'static dyn SessionIdProvider) {
        self.session_id_provider = Some(provider);
    }

    /// Set timestamp getter function (legacy, prefer set_timestamp_provider)
    pub fn set_timestamp_getter(&mut self, getter: fn() -> u32) {
        self.get_tmsp = Some(getter);
    }

    /// Set session ID getter function (legacy, prefer set_session_id_provider)
    pub fn set_session_id_getter(&mut self, getter: fn() -> u32) {
        self.get_sess_id = Some(getter);
    }

    // ========================================
    // Message Counter Management
    // ========================================

    /// Increment the message counter (wraps at 255)
    pub fn increment_counter(&mut self) {
        self.message_counter = self.message_counter.wrapping_add(1);
    }

    /// Reset the message counter to 0
    pub fn reset_counter(&mut self) {
        self.message_counter = 0;
    }

    /// Get the current message counter value
    pub fn get_counter(&self) -> u8 {
        self.message_counter
    }

    // ========================================
    // Other Configuration
    // ========================================

    /// Set byte order for multi-byte fields
    pub fn set_endian(&mut self, endian: DltEndian) {
        self.endian = endian;
    }

    // ========================================
    // Internal Field Accessors (for service builder)
    // ========================================

    /// Get the header type byte (internal use)
    #[doc(hidden)]
    pub fn get_header_htyp(&self) -> u8 {
        self.header_htyp
    }

    /// Get the endianness setting (internal use)
    #[doc(hidden)]
    pub fn get_endian(&self) -> &DltEndian {
        &self.endian
    }

    /// Get the timestamp provider (internal use)
    #[doc(hidden)]
    pub fn get_timestamp_provider(&self) -> Option<&'static dyn TimestampProvider> {
        self.timestamp_provider
    }

    /// Get the session ID provider (internal use)
    #[doc(hidden)]
    pub fn get_session_id_provider(&self) -> Option<&'static dyn SessionIdProvider> {
        self.session_id_provider
    }

    /// Get the serial header flag (internal use)
    #[doc(hidden)]
    pub fn has_serial_header(&self) -> bool {
        self.serial_header
    }

    /// Get the ECU ID (internal use)
    #[doc(hidden)]
    pub fn get_ecu_id(&self) -> &[u8; DLT_ID_SIZE] {
        self.ecu_id
    }

    /// Get the application ID (internal use)
    #[doc(hidden)]
    pub fn get_app_id(&self) -> &[u8; DLT_ID_SIZE] {
        self.app_id
    }

    /// Get the context ID (internal use)
    #[doc(hidden)]
    pub fn get_context_id(&self) -> &[u8; DLT_ID_SIZE] {
        self.context_id
    }

    // ========================================
    // Message Generation - Public API
    // ========================================

    /// Insert DLT header at the front of an existing payload buffer
    ///
    /// This method is useful when you've already written payload data to the start
    /// of the buffer and want to prepend the DLT headers.
    ///
    /// # Arguments
    /// * `buffer` - Buffer containing payload at the start; headers will be inserted at front
    /// * `payload_size` - Number of payload bytes already in the buffer
    /// * `arg_num` - Number of arguments in the payload
    /// * `log_level` - Log level for the message
    ///
    /// # Returns
    /// Total message size (headers + payload) on success
    ///
    /// # Note
    /// The existing payload will be moved backward in the buffer to make room for headers.
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

    /// Generate a complete DLT log message with payload
    ///
    /// This method generates headers and copies the payload into the buffer.
    ///
    /// # Arguments
    /// * `buffer` - Destination buffer for the complete message
    /// * `payload` - Payload data to include in the message
    /// * `log_level` - Log level (Fatal, Error, Warn, Info, Debug, Verbose)
    /// * `number_of_arguments` - Number of typed arguments in payload (for verbose mode)
    /// * `verbose` - Whether to use verbose mode (typed payload with metadata)
    ///
    /// # Returns
    /// Total message size on success
    ///
    /// # Example
    /// ```no_run
    /// use dlt_protocol::r19_11::*;
    ///
    /// let mut builder = DltMessageBuilder::new();
    /// let mut buffer = [0u8; 256];
    /// let str = "Hello, DLT!";
    /// let size = builder.generate_log_message_with_payload(
    ///     &mut buffer,
    ///     str.as_bytes(),
    ///     MtinTypeDltLog::DltLogInfo,
    ///     1,
    ///     true,
    /// ).unwrap();
    /// ```
    pub fn generate_log_message_with_payload(
        &mut self,
        buffer: &mut [u8],
        payload: &[u8],
        log_level: MtinTypeDltLog,
        number_of_arguments: u8,
        verbose: bool,
    ) -> Result<usize, DltError> {
        let header_size = self._generate_log_message_header_size();
        let serial_size = if self.serial_header { DLT_SERIAL_HEADER_SIZE } else { 0 };
        let payload_offset = serial_size + header_size;

        // Build payload using PayloadBuilder in verbose mode, or copy raw bytes in non-verbose mode
        let payload_size = if verbose {
            // Use PayloadBuilder to encode the payload with type information
            let mut payload_builder = PayloadBuilder::new(&mut buffer[payload_offset..]);
            
            // Convert payload bytes to string and add with type info
            let payload_str = core::str::from_utf8(payload)
                .map_err(|_| DltError::InvalidParameter)?;
            payload_builder.add_string(payload_str)
                .map_err(|_| DltError::BufferTooSmall)?;
            
            payload_builder.len()
        } else {
            // Non-verbose: copy raw payload bytes
            if payload_offset + payload.len() > buffer.len() {
                return Err(DltError::BufferTooSmall);
            }
            buffer[payload_offset..payload_offset + payload.len()].copy_from_slice(payload);
            payload.len()
        };

        let total_size = serial_size + header_size + payload_size;
        
        if buffer.len() < total_size {
            return Err(DltError::BufferTooSmall);
        }

        // Generate headers (this will write into buffer[0..payload_offset])
        let _header_bytes_written = self._generate_log_message(buffer, payload_size, log_level, number_of_arguments, verbose)?;

        Ok(total_size)
    }
    
    // ========================================
    // Message Generation - Internal Implementation
    // ========================================

    /// Internal method to generate DLT log message headers
    ///
    /// This method writes all headers (serial, standard, extra, extended) to the buffer.
    /// The message counter is automatically incremented after generation.
    ///
    /// # Note
    /// This is a public method for testing purposes, but should generally not be called
    /// directly. Use `generate_log_message_with_payload` or `insert_header_at_front` instead.
    ///
    /// # Returns  
    /// Number of header bytes written (excluding payload)
    pub fn _generate_log_message(
        &mut self,
        buffer: &mut [u8],
        payload_size: usize,
        log_level: MtinTypeDltLog,
        number_of_arguments: u8,
        verbose: bool,
    ) -> Result<usize, DltError> {
        let mut offset = 0;
        
        // Calculate message length (per DLT spec: excludes serial header)
        let header_size = self._generate_log_message_header_size();
        let len_field = (header_size + payload_size) as u16;
        
        let total_len = if self.serial_header {
            DLT_SERIAL_HEADER_SIZE + header_size + payload_size
        } else {
            header_size + payload_size
        };

        if buffer.len() < total_len {
            return Err(DltError::BufferTooSmall);
        }

        // ----------------------------------------
        // 1. Write Serial Header (optional)
        // ----------------------------------------
        if self.serial_header {
            buffer[offset..offset + DLT_SERIAL_HEADER_SIZE]
                .copy_from_slice(&DLT_SERIAL_HEADER_ARRAY);
            offset += DLT_SERIAL_HEADER_SIZE;
        }

        // ----------------------------------------
        // 2. Write Standard Header (4 bytes)
        // ----------------------------------------
        
        // HTYP byte: Update MSBF bit based on endianness
        let mut htyp = self.header_htyp;
        match self.endian {
            DltEndian::Big => htyp |= DltHeaderHtyp::MSBF_MASK,
            DltEndian::Little => htyp &= !DltHeaderHtyp::MSBF_MASK,
        }
        buffer[offset] = htyp;
        offset += 1;
        
        // Message Counter (auto-incremented after generation)
        buffer[offset] = self.message_counter;
        offset += 1;

        // Length field (excludes serial header, includes everything else)
        // Per DLT spec PRS_Dlt_00091: Always big-endian (network byte order)
        buffer[offset..offset + 2].copy_from_slice(&len_field.to_be_bytes());
        offset += 2;

        // ----------------------------------------
        // 3. Write Standard Header Extra Fields
        // ----------------------------------------
        
        // ECU ID (4 bytes, present if WEID flag set)
        if self.header_htyp & DltHeaderHtyp::WEID_MASK != 0 {
            buffer[offset..offset + DLT_ID_SIZE].copy_from_slice(self.ecu_id);
            offset += DLT_ID_SIZE;
        }

        // Session ID (4 bytes, present if WSID flag set)
        if self.header_htyp & DltHeaderHtyp::WSID_MASK != 0 {
            // Use dynamic provider if available
            if let Some(provider) = &self.session_id_provider {
                self.session_id = provider.get_session_id();
            }
            buffer[offset..offset + 4]
                .copy_from_slice(&convert_u32_to_bytes(self.session_id, &DltEndian::Big));
            offset += 4;
        }

        // Timestamp (4 bytes, present if WTMS flag set)
        if self.header_htyp & DltHeaderHtyp::WTMS_MASK != 0 {
            // Use dynamic provider if available
            if let Some(provider) = &self.timestamp_provider {
                self.timestamp = provider.get_timestamp();
            }
            buffer[offset..offset + 4]
                .copy_from_slice(&convert_u32_to_bytes(self.timestamp, &DltEndian::Big));
            offset += 4;
        }

        // ----------------------------------------
        // 4. Write Extended Header (10 bytes)
        // ----------------------------------------
        
        // MSIN byte: Encode verbose flag, message type, and log level
        let msin = encode_msin(
            verbose,
            MstpType::DltTypeLog.to_bits(),
            log_level.to_bits()
        );
        buffer[offset] = msin;
        offset += 1;

        // NOAR: Number of arguments
        buffer[offset] = number_of_arguments;
        offset += 1;

        // Application ID (4 bytes)
        buffer[offset..offset + DLT_ID_SIZE].copy_from_slice(self.app_id);
        offset += DLT_ID_SIZE;

        // Context ID (4 bytes)
        buffer[offset..offset + DLT_ID_SIZE].copy_from_slice(self.context_id);
        offset += DLT_ID_SIZE;

        // ----------------------------------------
        // 5. Post-generation: Increment counter
        // ----------------------------------------
        self.increment_counter();

        Ok(offset)
    }

    // ========================================
    // Size Calculation Helpers
    // ========================================

    /// Calculate the size of standard header extra fields based on HTYP flags
    fn _standard_header_extra_size(&self) -> usize {
        let mut size = 0;
        if self.header_htyp & DltHeaderHtyp::WEID_MASK != 0 {
            size += DLT_ID_SIZE; // ECU ID (4 bytes)
        }
        if self.header_htyp & DltHeaderHtyp::WSID_MASK != 0 {
            size += 4; // Session ID (4 bytes)
        }
        if self.header_htyp & DltHeaderHtyp::WTMS_MASK != 0 {
            size += 4; // Timestamp (4 bytes)
        }
        size
    }

    /// Calculate total header size (excludes serial header, excludes payload)
    fn _generate_log_message_header_size(&self) -> usize {
        DLT_STANDARD_HEADER_SIZE          // 4 bytes: HTYP, MCNT, LEN
            + self._standard_header_extra_size()  // 0-12 bytes: ECU ID, Session ID, Timestamp
            + DLT_EXTENDED_HEADER_SIZE     // 10 bytes: MSIN, NOAR, APID, CTID
    }
}

// ========================================
// Public Utility Functions
// ========================================

/// Convert a byte slice to a fixed-size DLT ID array (4 bytes)
///
/// If the input is shorter than 4 bytes, the remaining bytes are zero-padded.
/// If longer, only the first 4 bytes are used.
///
/// # Example
/// ```no_run
/// use dlt_protocol::r19_11::to_dlt_id_array;
///
/// let id = to_dlt_id_array(b"APP");   // Returns [b'A', b'P', b'P', 0]
/// let id2 = to_dlt_id_array(b"TEST"); // Returns [b'T', b'E', b'S', b'T']
/// ```
#[inline]
pub fn to_dlt_id_array(id: &[u8]) -> [u8; DLT_ID_SIZE] {
    let mut array = [0u8; DLT_ID_SIZE];
    let len = core::cmp::min(id.len(), DLT_ID_SIZE);
    array[..len].copy_from_slice(&id[..len]);
    array
}

// ========================================
// Internal Endian Conversion Helpers
// ========================================

/// Convert u32 to byte array based on endianness
#[inline]
fn convert_u32_to_bytes(value: u32, endian: &DltEndian) -> [u8; 4] {
    match endian {
        DltEndian::Big => value.to_be_bytes(),
        DltEndian::Little => value.to_le_bytes(),
    }
}

/// Convert u16 to byte array based on endianness
#[inline]
fn convert_u16_to_bytes(value: u16, endian: &DltEndian) -> [u8; 2] {
    match endian {
        DltEndian::Big => value.to_be_bytes(),
        DltEndian::Little => value.to_le_bytes(),
    }
}
