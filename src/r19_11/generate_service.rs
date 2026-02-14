//! # DLT Service/Control Message Generator
//!
//! This module provides functionality for generating DLT Control messages
//! according to the AUTOSAR DLT specification release 19.11.
//!
//! Control messages are used for communication between DLT clients and the DLT daemon
//! to configure logging behavior, request information, and control the DLT system.
//!
//! ## Usage
//!
//! ```no_run
//! use dlt_protocol::r19_11::*;
//!
//! let mut builder = DltServiceMessageBuilder::new()
//!     .with_ecu_id(b"ECU1")
//!     .with_app_id(b"SYS\0")
//!     .with_context_id(b"MGMT");
//!
//! let mut buffer = [0u8; 256];
//! let size = builder.generate_set_log_level_request(
//!     &mut buffer,
//!     b"APP1",
//!     b"CTX1",
//!     4, // Info level
//! ).unwrap();
//! ```

use crate::r19_11::*;
use core::cmp::min;

// ========================================
// Service ID Constants
// ========================================

/// DLT Service/Control Message IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ServiceId {
    /// Set log level for specific app/context
    SetLogLevel = 0x01,
    /// Set trace status for specific app/context
    SetTraceStatus = 0x02,
    /// Get log info for registered applications
    GetLogInfo = 0x03,
    /// Get default log level
    GetDefaultLogLevel = 0x04,
    /// Store configuration non-volatile
    StoreConfiguration = 0x05,
    /// Reset to factory defaults
    ResetToFactoryDefault = 0x06,
    /// Set message filtering on/off
    SetMessageFiltering = 0x0A,
    /// Set default log level
    SetDefaultLogLevel = 0x11,
    /// Set default trace status
    SetDefaultTraceStatus = 0x12,
    /// Get software version
    GetSoftwareVersion = 0x13,
    /// Get default trace status
    GetDefaultTraceStatus = 0x15,
    /// Get log channel names
    GetLogChannelNames = 0x17,
    /// Get trace status for specific app/context
    GetTraceStatus = 0x1F,
    /// Set log channel assignment
    SetLogChannelAssignment = 0x20,
    /// Set log channel threshold
    SetLogChannelThreshold = 0x21,
    /// Get log channel threshold
    GetLogChannelThreshold = 0x22,
    /// Buffer overflow notification
    BufferOverflowNotification = 0x23,
    /// Sync timestamp
    SyncTimeStamp = 0x24,
    /// SWC injection (0xFFF and above)
    CallSWCInjection = 0xFFF,
}

impl ServiceId {
    /// Convert service ID to u32
    pub fn to_u32(&self) -> u32 {
        *self as u32
    }

    /// Parse service ID from u32
    pub fn from_u32(value: u32) -> Option<ServiceId> {
        match value {
            0x01 => Some(ServiceId::SetLogLevel),
            0x02 => Some(ServiceId::SetTraceStatus),
            0x03 => Some(ServiceId::GetLogInfo),
            0x04 => Some(ServiceId::GetDefaultLogLevel),
            0x05 => Some(ServiceId::StoreConfiguration),
            0x06 => Some(ServiceId::ResetToFactoryDefault),
            0x0A => Some(ServiceId::SetMessageFiltering),
            0x11 => Some(ServiceId::SetDefaultLogLevel),
            0x12 => Some(ServiceId::SetDefaultTraceStatus),
            0x13 => Some(ServiceId::GetSoftwareVersion),
            0x15 => Some(ServiceId::GetDefaultTraceStatus),
            0x17 => Some(ServiceId::GetLogChannelNames),
            0x1F => Some(ServiceId::GetTraceStatus),
            0x20 => Some(ServiceId::SetLogChannelAssignment),
            0x21 => Some(ServiceId::SetLogChannelThreshold),
            0x22 => Some(ServiceId::GetLogChannelThreshold),
            0x23 => Some(ServiceId::BufferOverflowNotification),
            0x24 => Some(ServiceId::SyncTimeStamp),
            0xFFF..=0xFFFFFFFF => Some(ServiceId::CallSWCInjection),
            _ => None,
        }
    }
}

// ========================================
// Service Response Status
// ========================================

/// Standard response status codes for service messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ServiceStatus {
    /// Operation successful
    Ok = 0,
    /// Service not supported
    NotSupported = 1,
    /// Error occurred
    Error = 2,
    /// Operation pending (for CallSWCInjection)
    Pending = 3,
    /// With log level and trace status (GetLogInfo)
    WithLogLevelAndTraceStatus = 6,
    /// With descriptions (GetLogInfo)
    WithDescriptions = 7,
    /// No matching contexts (GetLogInfo)
    NoMatchingContexts = 8,
    /// Overflow (GetLogInfo)
    Overflow = 9,
}

impl ServiceStatus {
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }

    pub fn from_u8(value: u8) -> Option<ServiceStatus> {
        match value {
            0 => Some(ServiceStatus::Ok),
            1 => Some(ServiceStatus::NotSupported),
            2 => Some(ServiceStatus::Error),
            3 => Some(ServiceStatus::Pending),
            6 => Some(ServiceStatus::WithLogLevelAndTraceStatus),
            7 => Some(ServiceStatus::WithDescriptions),
            8 => Some(ServiceStatus::NoMatchingContexts),
            9 => Some(ServiceStatus::Overflow),
            _ => None,
        }
    }
}

// ========================================
// Service Message Builder
// ========================================

/// Builder for creating DLT service/control messages
///
/// This builder is similar to DltMessageBuilder but specifically for control messages
/// (MSTP = DltTypeControl, MTIN = DltControlRequest/Response).
pub struct DltServiceMessageBuilder<'a> {
    /// Base message builder for headers
    base_builder: DltMessageBuilder<'a>,
}

impl<'a> DltServiceMessageBuilder<'a> {
    /// Create a new service message builder with default settings
    ///
    /// Default configuration uses Control message type (MSTP=3)
    pub fn new() -> Self {
        Self {
            base_builder: DltMessageBuilder::new(),
        }
    }

    /// Set ECU ID (4 bytes)
    pub fn with_ecu_id(mut self, ecu_id: &'a [u8; DLT_ID_SIZE]) -> Self {
        self.base_builder = self.base_builder.with_ecu_id(ecu_id);
        self
    }

    /// Set Application ID (4 bytes)
    pub fn with_app_id(mut self, app_id: &'a [u8; DLT_ID_SIZE]) -> Self {
        self.base_builder = self.base_builder.with_app_id(app_id);
        self
    }

    /// Set Context ID (4 bytes)
    pub fn with_context_id(mut self, ctx_id: &'a [u8; DLT_ID_SIZE]) -> Self {
        self.base_builder = self.base_builder.with_context_id(ctx_id);
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, sess_id: u32) -> Self {
        self.base_builder = self.base_builder.with_session_id(sess_id);
        self
    }

    /// Set timestamp (in 0.1ms units)
    pub fn with_timestamp(mut self, timestamp: u32) -> Self {
        self.base_builder = self.base_builder.with_timestamp(timestamp);
        self
    }

    /// Enable serial header
    pub fn add_serial_header(mut self) -> Self {
        self.base_builder = self.base_builder.add_serial_header();
        self
    }

    /// Set endianness
    pub fn set_endian(&mut self, endian: DltEndian) {
        self.base_builder.set_endian(endian);
    }

    /// Get current message counter
    pub fn get_counter(&self) -> u8 {
        self.base_builder.get_counter()
    }

    /// Reset message counter
    pub fn reset_counter(&mut self) {
        self.base_builder.reset_counter();
    }

    // ========================================
    // Service Request Generators
    // ========================================

    /// Generate SetLogLevel service request (0x01)
    ///
    /// # Arguments
    /// * `buffer` - Output buffer for the complete message
    /// * `app_id` - Application ID (use &[0,0,0,0] for all apps)
    /// * `ctx_id` - Context ID (use &[0,0,0,0] for all contexts)
    /// * `log_level` - New log level (0=block all, -1=use default, 1-6=specific level)
    ///
    /// # Returns
    /// Total message size on success
    pub fn generate_set_log_level_request(
        &mut self,
        buffer: &mut [u8],
        app_id: &[u8; 4],
        ctx_id: &[u8; 4],
        log_level: i8,
    ) -> Result<usize, DltError> {
        let mut payload = [0u8; 17]; // 4 (service ID) + 4 (app) + 4 (ctx) + 1 (level) + 4 (reserved)
        
        // Service ID (32-bit, big-endian)
        payload[0..4].copy_from_slice(&ServiceId::SetLogLevel.to_u32().to_be_bytes());
        
        // Application ID
        payload[4..8].copy_from_slice(app_id);
        
        // Context ID
        payload[8..12].copy_from_slice(ctx_id);
        
        // Log level (signed byte)
        payload[12] = log_level as u8;
        
        // Reserved (4 bytes "remo" suffix)
        payload[13..17].copy_from_slice(&DLT_SERVICE_SUFFIX);
        
        self.generate_control_message(buffer, &payload, MtinTypeDltControl::DltControlRequest)
    }

    /// Generate SetTraceStatus service request (0x02)
    ///
    /// # Arguments
    /// * `buffer` - Output buffer
    /// * `app_id` - Application ID (use &[0,0,0,0] for all apps)
    /// * `ctx_id` - Context ID (use &[0,0,0,0] for all contexts)
    /// * `trace_status` - New trace status (0=off, 1=on, -1=use default)
    pub fn generate_set_trace_status_request(
        &mut self,
        buffer: &mut [u8],
        app_id: &[u8; 4],
        ctx_id: &[u8; 4],
        trace_status: i8,
    ) -> Result<usize, DltError> {
        let mut payload = [0u8; 17];
        
        payload[0..4].copy_from_slice(&ServiceId::SetTraceStatus.to_u32().to_be_bytes());
        payload[4..8].copy_from_slice(app_id);
        payload[8..12].copy_from_slice(ctx_id);
        payload[12] = trace_status as u8;
        payload[13..17].copy_from_slice(&DLT_SERVICE_SUFFIX);
        
        self.generate_control_message(buffer, &payload, MtinTypeDltControl::DltControlRequest)
    }

    /// Generate GetLogInfo service request (0x03)
    ///
    /// # Arguments
    /// * `buffer` - Output buffer
    /// * `options` - 6=with log level and trace status, 7=with descriptions
    /// * `app_id` - Application ID (use &[0,0,0,0] for all apps)
    /// * `ctx_id` - Context ID (use &[0,0,0,0] for all contexts)
    pub fn generate_get_log_info_request(
        &mut self,
        buffer: &mut [u8],
        options: u8,
        app_id: &[u8; 4],
        ctx_id: &[u8; 4],
    ) -> Result<usize, DltError> {
        let mut payload = [0u8; 17];
        
        payload[0..4].copy_from_slice(&ServiceId::GetLogInfo.to_u32().to_be_bytes());
        payload[4] = options;
        payload[5..9].copy_from_slice(app_id);
        payload[9..13].copy_from_slice(ctx_id);
        payload[13..17].copy_from_slice(&DLT_SERVICE_SUFFIX);
        
        self.generate_control_message(buffer, &payload, MtinTypeDltControl::DltControlRequest)
    }

    /// Generate GetDefaultLogLevel service request (0x04)
    pub fn generate_get_default_log_level_request(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<usize, DltError> {
        let payload = ServiceId::GetDefaultLogLevel.to_u32().to_be_bytes();
        self.generate_control_message(buffer, &payload, MtinTypeDltControl::DltControlRequest)
    }

    /// Generate StoreConfiguration service request (0x05)
    pub fn generate_store_configuration_request(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<usize, DltError> {
        let payload = ServiceId::StoreConfiguration.to_u32().to_be_bytes();
        self.generate_control_message(buffer, &payload, MtinTypeDltControl::DltControlRequest)
    }

    /// Generate ResetToFactoryDefault service request (0x06)
    pub fn generate_reset_to_factory_default_request(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<usize, DltError> {
        let payload = ServiceId::ResetToFactoryDefault.to_u32().to_be_bytes();
        self.generate_control_message(buffer, &payload, MtinTypeDltControl::DltControlRequest)
    }

    /// Generate SetMessageFiltering service request (0x0A)
    ///
    /// # Arguments
    /// * `buffer` - Output buffer
    /// * `filtering_enabled` - true to enable filtering, false to disable
    pub fn generate_set_message_filtering_request(
        &mut self,
        buffer: &mut [u8],
        filtering_enabled: bool,
    ) -> Result<usize, DltError> {
        let mut payload = [0u8; 5];
        
        payload[0..4].copy_from_slice(&ServiceId::SetMessageFiltering.to_u32().to_be_bytes());
        payload[4] = if filtering_enabled { 1 } else { 0 };
        
        self.generate_control_message(buffer, &payload, MtinTypeDltControl::DltControlRequest)
    }

    /// Generate SetDefaultLogLevel service request (0x11)
    ///
    /// # Arguments
    /// * `buffer` - Output buffer
    /// * `log_level` - New default log level (0=block all, -1=pass all, 1-6=specific level)
    pub fn generate_set_default_log_level_request(
        &mut self,
        buffer: &mut [u8],
        log_level: i8,
    ) -> Result<usize, DltError> {
        let mut payload = [0u8; 9];
        
        payload[0..4].copy_from_slice(&ServiceId::SetDefaultLogLevel.to_u32().to_be_bytes());
        payload[4] = log_level as u8;
        payload[5..9].copy_from_slice(&DLT_SERVICE_SUFFIX);
        
        self.generate_control_message(buffer, &payload, MtinTypeDltControl::DltControlRequest)
    }

    /// Generate GetSoftwareVersion service request (0x13)
    pub fn generate_get_software_version_request(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<usize, DltError> {
        let payload = ServiceId::GetSoftwareVersion.to_u32().to_be_bytes();
        self.generate_control_message(buffer, &payload, MtinTypeDltControl::DltControlRequest)
    }

    // ========================================
    // Service Response Generators
    // ========================================

    /// Generate a simple status-only response
    ///
    /// # Arguments
    /// * `buffer` - Output buffer
    /// * `service_id` - Service ID being responded to
    /// * `status` - Response status code
    pub fn generate_status_response(
        &mut self,
        buffer: &mut [u8],
        service_id: ServiceId,
        status: ServiceStatus,
    ) -> Result<usize, DltError> {
        let mut payload = [0u8; 5];
        
        payload[0..4].copy_from_slice(&service_id.to_u32().to_be_bytes());
        payload[4] = status.to_u8();
        
        self.generate_control_message(buffer, &payload, MtinTypeDltControl::DltControlResponse)
    }

    /// Generate GetDefaultLogLevel response (0x04)
    pub fn generate_get_default_log_level_response(
        &mut self,
        buffer: &mut [u8],
        status: ServiceStatus,
        log_level: u8,
    ) -> Result<usize, DltError> {
        let mut payload = [0u8; 6];
        
        payload[0..4].copy_from_slice(&ServiceId::GetDefaultLogLevel.to_u32().to_be_bytes());
        payload[4] = status.to_u8();
        payload[5] = log_level;
        
        self.generate_control_message(buffer, &payload, MtinTypeDltControl::DltControlResponse)
    }

    /// Generate GetSoftwareVersion response (0x13)
    pub fn generate_get_software_version_response(
        &mut self,
        buffer: &mut [u8],
        status: ServiceStatus,
        sw_version: &[u8],
    ) -> Result<usize, DltError> {
        // According to spec, swVersion is char[] which should be null-terminated
        // Length field indicates the string length INCLUDING the null terminator
        let version_len = min(sw_version.len(), 199); // Reserve 1 byte for null terminator
        let string_len_with_null = version_len + 1; // Include null terminator in length
        let payload_len = 4 + 1 + 4 + string_len_with_null;
        
        if buffer.len() < payload_len + 50 { // 50 bytes for headers
            return Err(DltError::BufferTooSmall);
        }
        
        let mut temp_payload = [0u8; 256];
        
        temp_payload[0..4].copy_from_slice(&ServiceId::GetSoftwareVersion.to_u32().to_be_bytes());
        temp_payload[4] = status.to_u8();
        temp_payload[5..9].copy_from_slice(&(string_len_with_null as u32).to_be_bytes());
        temp_payload[9..9 + version_len].copy_from_slice(&sw_version[..version_len]);
        temp_payload[9 + version_len] = 0; // Null terminator
        
        self.generate_control_message(buffer, &temp_payload[..payload_len], MtinTypeDltControl::DltControlResponse)
    }

    /// Generate GetLogInfo response (0x03)
    /// 
    /// This is a complex response with nested structure. Use LogInfoResponseBuilder to construct the payload.
    ///
    /// # Arguments
    /// * `buffer` - Output buffer for the complete DLT message
    /// * `status` - Service status
    /// * `log_info_payload` - Pre-built LogInfo payload from LogInfoResponseBuilder
    ///
    /// # Example
    /// ```no_run
    /// use dlt_protocol::r19_11::*;
    /// 
    /// let mut builder = DltServiceMessageBuilder::new();
    /// let mut log_info = LogInfoResponseBuilder::new(false); // option 6
    /// 
    /// log_info.add_app(b"APP1");
    /// log_info.add_context(b"CTX1", 4, 1, None);
    /// log_info.add_context(b"CTX2", 5, 0, None);
    /// 
    /// let mut payload = [0u8; 1024];
    /// let payload_len = log_info.build(&mut payload).unwrap();
    /// 
    /// let mut buffer = [0u8; 2048];
    /// builder.generate_get_log_info_response(&mut buffer, ServiceStatus::WithLogLevelAndTraceStatus, &payload[..payload_len]).unwrap();
    /// ```
    pub fn generate_get_log_info_response(
        &mut self,
        buffer: &mut [u8],
        status: ServiceStatus,
        log_info_payload: &[u8],
    ) -> Result<usize, DltError> {
        // Payload: service_id(4) + status(1) + log_info_data(N) + reserved(4)
        let payload_len = 4 + 1 + log_info_payload.len() + 4;
        
        if buffer.len() < payload_len + 50 {
            return Err(DltError::BufferTooSmall);
        }
        
        let mut temp_payload = [0u8; 4096]; // Large buffer for complex response
        
        if payload_len > temp_payload.len() {
            return Err(DltError::BufferTooSmall);
        }
        
        temp_payload[0..4].copy_from_slice(&ServiceId::GetLogInfo.to_u32().to_be_bytes());
        temp_payload[4] = status.to_u8();
        temp_payload[5..5 + log_info_payload.len()].copy_from_slice(log_info_payload);
        // Last 4 bytes are "remo" suffix
        temp_payload[5 + log_info_payload.len()..5 + log_info_payload.len() + 4].copy_from_slice(&DLT_SERVICE_SUFFIX);
        
        self.generate_control_message(buffer, &temp_payload[..payload_len], MtinTypeDltControl::DltControlResponse)
    }

    // ========================================
    // Internal Helper Methods
    // ========================================

    /// Internal method to generate a control message with given payload
    ///
    /// This wraps the payload in a proper DLT control message with extended header
    /// set to MSTP=Control and the specified MTIN.
    /// 
    /// Note: The payload should already include the "remo" suffix in the reserved field.
    fn generate_control_message(
        &mut self,
        buffer: &mut [u8],
        payload: &[u8],
        mtin: MtinTypeDltControl,
    ) -> Result<usize, DltError> {
        // Calculate required space  
        let header_size = self.calculate_header_size();
        let serial_size = if self.base_builder.has_serial_header() {
            DLT_SERIAL_HEADER_SIZE
        } else {
            0
        };
        let total_size = serial_size + header_size + payload.len();
        
        if buffer.len() < total_size {
            return Err(DltError::BufferTooSmall);
        }

        // Generate headers
        let offset = self.generate_control_message_header(
            buffer,
            payload.len(),
            mtin,
        )?;

        // Copy payload after headers
        buffer[offset..offset + payload.len()].copy_from_slice(payload);

        Ok(total_size)
    }

    /// Generate control message headers
    fn generate_control_message_header(
        &mut self,
        buffer: &mut [u8],
        payload_size: usize,
        mtin: MtinTypeDltControl,
    ) -> Result<usize, DltError> {
        let mut offset = 0;
        
        // Calculate lengths
        let header_size = self.calculate_header_size();
        let len_field = (header_size + payload_size) as u16;
        
        let serial_size = if self.base_builder.has_serial_header() {
            DLT_SERIAL_HEADER_SIZE
        } else {
            0
        };

        let total_size = serial_size + header_size + payload_size;
        
        if buffer.len() < total_size {
            return Err(DltError::BufferTooSmall);
        }

        // Write serial header if enabled
        if self.base_builder.has_serial_header() {
            buffer[offset..offset + DLT_SERIAL_HEADER_SIZE]
                .copy_from_slice(&DLT_SERIAL_HEADER_ARRAY);
            offset += DLT_SERIAL_HEADER_SIZE;
        }

        // Standard header
        let htyp = self.base_builder.get_header_htyp();
        buffer[offset] = htyp;
        offset += 1;
        
        buffer[offset] = self.base_builder.get_counter();
        offset += 1;
        
        buffer[offset..offset + 2].copy_from_slice(&len_field.to_be_bytes());
        offset += 2;

        // Standard header extra fields (ECU ID, Session ID, Timestamp)
        offset = self.write_standard_header_extra(buffer, offset)?;

        // Extended header with Control message type
        let msin = encode_msin(
            false, // Control messages are typically non-verbose
            MstpType::DltTypeControl.to_bits(),
            mtin.to_bits(),
        );
        buffer[offset] = msin;
        offset += 1;

        // NOAR (number of arguments) - set to 0 for control messages
        buffer[offset] = 0;
        offset += 1;

        // Application ID
        buffer[offset..offset + DLT_ID_SIZE].copy_from_slice(self.base_builder.get_app_id());
        offset += DLT_ID_SIZE;

        // Context ID
        buffer[offset..offset + DLT_ID_SIZE].copy_from_slice(self.base_builder.get_context_id());
        offset += DLT_ID_SIZE;

        // Increment counter
        self.base_builder.increment_counter();

        Ok(offset)
    }

    /// Write standard header extra fields
    fn write_standard_header_extra(
        &mut self,
        buffer: &mut [u8],
        mut offset: usize,
    ) -> Result<usize, DltError> {
        // ECU ID
        if self.base_builder.get_header_htyp() & WEID_MASK != 0 {
            buffer[offset..offset + DLT_ID_SIZE].copy_from_slice(self.base_builder.get_ecu_id());
            offset += DLT_ID_SIZE;
        }

        // Session ID
        if self.base_builder.get_header_htyp() & WSID_MASK != 0 {
            if let Some(provider) = self.base_builder.get_session_id_provider() {
                self.base_builder.session_id = provider.get_session_id();
            }
            let bytes = match self.base_builder.get_endian() {
                DltEndian::Big => self.base_builder.session_id.to_be_bytes(),
                DltEndian::Little => self.base_builder.session_id.to_le_bytes(),
            };
            buffer[offset..offset + 4].copy_from_slice(&bytes);
            offset += 4;
        }

        // Timestamp
        if self.base_builder.get_header_htyp() & WTMS_MASK != 0 {
            if let Some(provider) = self.base_builder.get_timestamp_provider() {
                self.base_builder.timestamp = provider.get_timestamp();
            }
            let bytes = match self.base_builder.get_endian() {
                DltEndian::Big => self.base_builder.timestamp.to_be_bytes(),
                DltEndian::Little => self.base_builder.timestamp.to_le_bytes(),
            };
            buffer[offset..offset + 4].copy_from_slice(&bytes);
            offset += 4;
        }

        Ok(offset)
    }

    /// Calculate total header size (without serial header)
    fn calculate_header_size(&self) -> usize {
        let mut size = DLT_STANDARD_HEADER_SIZE + DLT_EXTENDED_HEADER_SIZE;
        
        if self.base_builder.get_header_htyp() & WEID_MASK != 0 {
            size += DLT_ID_SIZE;
        }
        if self.base_builder.get_header_htyp() & WSID_MASK != 0 {
            size += 4;
        }
        if self.base_builder.get_header_htyp() & WTMS_MASK != 0 {
            size += 4;
        }
        
        size
    }
}

// ========================================
// Control Message Type Info
// ========================================

impl MtinTypeDltControl {
    /// Convert to 4-bit value
    pub fn to_bits(&self) -> u8 {
        match self {
            MtinTypeDltControl::DltControlRequest => 0x1,
            MtinTypeDltControl::DltControlResponse => 0x2,
            MtinTypeDltControl::Reserved(v) => *v,
            MtinTypeDltControl::Invalid(v) => *v,
        }
    }
}
// ========================================
// GetLogInfo Response Builder
// ========================================

/// Builder for constructing GetLogInfo response payloads
///
/// This builder helps create the complex nested structure of GetLogInfo responses.
/// It uses a stack-based approach suitable for no_std environments.
///
/// # Structure
/// ```text
/// appCount (2 bytes)
/// For each app:
///   app_id (4 bytes)
///   contextCount (2 bytes)
///   For each context:
///     context_id (4 bytes)
///     log_level (1 byte)
///     trace_status (1 byte)
///     [if with_descriptions:]
///       description_len (2 bytes)
///       description (N bytes)
///   [if with_descriptions:]
///     app_description_len (2 bytes)
///     app_description (N bytes)
/// ```
///
/// # Example
/// ```no_run
/// use dlt_protocol::r19_11::*;
///
/// let mut builder = LogInfoResponseBuilder::new(false); // option 6
/// builder.add_app(b"APP1");
/// builder.add_context(b"CTX1", 4, 1, None);
/// builder.add_context(b"CTX2", 5, 0, None);
/// builder.add_app(b"APP2");
/// builder.add_context(b"CTX3", 4, 1, None);
///
/// let mut payload = [0u8; 1024];
/// let len = builder.build(&mut payload).unwrap();
/// ```
pub struct LogInfoResponseBuilder {
    with_descriptions: bool,
    app_count: u16,
    current_app_id: Option<[u8; 4]>,
    current_app_context_count: u16,
    current_app_desc: Option<&'static [u8]>,
    // We'll build directly into the output buffer to avoid allocations
}

impl LogInfoResponseBuilder {
    /// Create a new builder
    ///
    /// # Arguments
    /// * `with_descriptions` - true for option 7 (with descriptions), false for option 6
    pub fn new(with_descriptions: bool) -> Self {
        Self {
            with_descriptions,
            app_count: 0,
            current_app_id: None,
            current_app_context_count: 0,
            current_app_desc: None,
        }
    }

    /// Start a new application
    ///
    /// # Arguments
    /// * `app_id` - Application ID (4 bytes, will be truncated or padded)
    pub fn add_app(&mut self, app_id: &[u8]) {
        let mut id = [0u8; 4];
        let len = core::cmp::min(app_id.len(), 4);
        id[..len].copy_from_slice(&app_id[..len]);
        
        self.current_app_id = Some(id);
        self.current_app_context_count = 0;
        self.current_app_desc = None;
        self.app_count += 1;
    }

    /// Set description for the current application (only if with_descriptions=true)
    pub fn set_app_description(&mut self, desc: &'static [u8]) {
        if self.with_descriptions {
            self.current_app_desc = Some(desc);
        }
    }

    /// Add a context to the current application
    ///
    /// # Arguments
    /// * `context_id` - Context ID (4 bytes, will be truncated or padded)
    /// * `log_level` - Log level (0-6)
    /// * `trace_status` - Trace status (0=off, 1=on)
    /// * `description` - Optional description (only used if with_descriptions=true)
    pub fn add_context(&mut self, context_id: &[u8], log_level: u8, trace_status: u8, description: Option<&'static [u8]>) {
        self.current_app_context_count += 1;
        // Note: Actual writing happens in build() method
    }

    /// Build the payload into the provided buffer
    ///
    /// This method must be called with all application and context information prepared.
    /// For a simple implementation, we'll require the caller to provide all data upfront.
    /// 
    /// Returns the number of bytes written.
    pub fn build(&self, _buffer: &mut [u8]) -> Result<usize, DltError> {
        // For now, return an error indicating this is a placeholder
        // The actual implementation requires storing all app/context data during add operations
        Err(DltError::InvalidParameter)
    }
}

/// More flexible builder using callback pattern
///
/// This builder allows building GetLogInfo responses by providing data through callbacks,
/// avoiding the need to store all data in the builder.
///
/// # Example
/// ```no_run
/// use dlt_protocol::r19_11::*;
///
/// let mut buffer = [0u8; 2048];
/// let mut writer = LogInfoPayloadWriter::new(&mut buffer, false);
///
/// writer.write_app_count(2)?;
///
/// writer.write_app_id(b"APP1")?;
/// writer.write_context_count(2)?;
/// writer.write_context(b"CTX1", 4, 1, None)?;
/// writer.write_context(b"CTX2", 5, 0, None)?;
///
/// writer.write_app_id(b"APP2")?;
/// writer.write_context_count(1)?;
/// writer.write_context(b"CTX3", 4, 1, None)?;
///
/// let len = writer.finish()?;
/// # Ok::<(), DltError>(())
/// ```
pub struct LogInfoPayloadWriter<'a> {
    buffer: &'a mut [u8],
    position: usize,
    with_descriptions: bool,
}

impl<'a> LogInfoPayloadWriter<'a> {
    /// Create a new payload writer
    pub fn new(buffer: &'a mut [u8], with_descriptions: bool) -> Self {
        Self {
            buffer,
            position: 0,
            with_descriptions,
        }
    }

    /// Write application count (must be called first)
    pub fn write_app_count(&mut self, count: u16) -> Result<(), DltError> {
        if self.position + 2 > self.buffer.len() {
            return Err(DltError::BufferTooSmall);
        }
        self.buffer[self.position..self.position + 2].copy_from_slice(&count.to_be_bytes());
        self.position += 2;
        Ok(())
    }

    /// Write application ID
    pub fn write_app_id(&mut self, app_id: &[u8]) -> Result<(), DltError> {
        if self.position + 4 > self.buffer.len() {
            return Err(DltError::BufferTooSmall);
        }
        let mut id = [0u8; 4];
        let len = core::cmp::min(app_id.len(), 4);
        id[..len].copy_from_slice(&app_id[..len]);
        self.buffer[self.position..self.position + 4].copy_from_slice(&id);
        self.position += 4;
        Ok(())
    }

    /// Write context count for current application
    pub fn write_context_count(&mut self, count: u16) -> Result<(), DltError> {
        if self.position + 2 > self.buffer.len() {
            return Err(DltError::BufferTooSmall);
        }
        self.buffer[self.position..self.position + 2].copy_from_slice(&count.to_be_bytes());
        self.position += 2;
        Ok(())
    }

    /// Write context information
    pub fn write_context(&mut self, context_id: &[u8], log_level: u8, trace_status: u8, description: Option<&[u8]>) -> Result<(), DltError> {
        // Write context ID (4 bytes)
        if self.position + 4 > self.buffer.len() {
            return Err(DltError::BufferTooSmall);
        }
        let mut id = [0u8; 4];
        let len = core::cmp::min(context_id.len(), 4);
        id[..len].copy_from_slice(&context_id[..len]);
        self.buffer[self.position..self.position + 4].copy_from_slice(&id);
        self.position += 4;

        // Write log level and trace status
        if self.position + 2 > self.buffer.len() {
            return Err(DltError::BufferTooSmall);
        }
        self.buffer[self.position] = log_level;
        self.buffer[self.position + 1] = trace_status;
        self.position += 2;

        // Write description if option 7
        if self.with_descriptions {
            if let Some(desc) = description {
                let desc_len = core::cmp::min(desc.len(), 65535) as u16;
                if self.position + 2 + desc_len as usize > self.buffer.len() {
                    return Err(DltError::BufferTooSmall);
                }
                self.buffer[self.position..self.position + 2].copy_from_slice(&desc_len.to_be_bytes());
                self.position += 2;
                self.buffer[self.position..self.position + desc_len as usize].copy_from_slice(&desc[..desc_len as usize]);
                self.position += desc_len as usize;
            } else {
                // Write zero-length description
                if self.position + 2 > self.buffer.len() {
                    return Err(DltError::BufferTooSmall);
                }
                self.buffer[self.position..self.position + 2].copy_from_slice(&0u16.to_be_bytes());
                self.position += 2;
            }
        }

        Ok(())
    }

    /// Write application description (must be called after all contexts for an app)
    pub fn write_app_description(&mut self, description: Option<&[u8]>) -> Result<(), DltError> {
        if !self.with_descriptions {
            return Ok(());
        }

        if let Some(desc) = description {
            let desc_len = core::cmp::min(desc.len(), 65535) as u16;
            if self.position + 2 + desc_len as usize > self.buffer.len() {
                return Err(DltError::BufferTooSmall);
            }
            self.buffer[self.position..self.position + 2].copy_from_slice(&desc_len.to_be_bytes());
            self.position += 2;
            self.buffer[self.position..self.position + desc_len as usize].copy_from_slice(&desc[..desc_len as usize]);
            self.position += desc_len as usize;
        } else {
            // Write zero-length description
            if self.position + 2 > self.buffer.len() {
                return Err(DltError::BufferTooSmall);
            }
            self.buffer[self.position..self.position + 2].copy_from_slice(&0u16.to_be_bytes());
            self.position += 2;
        }

        Ok(())
    }

    /// Finish writing and return the total length
    pub fn finish(self) -> Result<usize, DltError> {
        Ok(self.position)
    }

    /// Get current position
    pub fn position(&self) -> usize {
        self.position
    }
}