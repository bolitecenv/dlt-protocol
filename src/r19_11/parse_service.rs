//! # DLT Service/Control Message Parser
//!
//! This module provides functionality for parsing DLT Control message payloads
//! according to the AUTOSAR DLT specification release 19.11.
//!
//! ## Overview
//!
//! Service messages (also called Control messages) are used for runtime configuration
//! and control of the DLT daemon. They have MSTP=DltTypeControl.
//!
//! ## Message Structure
//!
//! ```text
//! Service Message Payload:
//! ┌────────────────┬─────────────────────────┐
//! │ Service ID     │ Parameters              │
//! │ (4 bytes, BE)  │ (variable, service-specific) │
//! └────────────────┴─────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```no_run
//! use dlt_protocol::r19_11::*;
//!
//! // Parse a DLT message
//! let data: &[u8] = &[/* DLT packet bytes */];
//! let mut parser = DltHeaderParser::new(data);
//! let message = parser.parse_message().unwrap();
//!
//! // If it's a control message, parse the service payload
//! if let Some(ext_hdr) = message.extended_header {
//!     if matches!(ext_hdr.message_type(), MstpType::DltTypeControl) {
//!         let service_parser = DltServiceParser::new(message.payload);
//!         if let Ok(service_id) = service_parser.parse_service_id() {
//!             println!("Service ID: {:?}", service_id);
//!         }
//!     }
//! }
//! ```

use crate::r19_11::*;

// ========================================
// GetLogInfo Data Structures
// ========================================

/// Context information in GetLogInfo response
#[derive(Debug, Clone)]
pub struct LogInfoContext<'a> {
    /// Context ID (4 bytes)
    pub context_id: [u8; 4],
    /// Log level (0-6)
    pub log_level: u8,
    /// Trace status (0=off, 1=on)
    pub trace_status: u8,
    /// Context description (only if option=7)
    pub description: Option<&'a [u8]>,
}

/// Application information in GetLogInfo response
#[derive(Debug, Clone)]
pub struct LogInfoApp<'a> {
    /// Application ID (4 bytes)
    pub app_id: [u8; 4],
    /// List of contexts for this application
    pub contexts: &'a [LogInfoContext<'a>],
    /// Application description (only if option=7)
    pub description: Option<&'a [u8]>,
}

// ========================================
// Service Message Parser
// ========================================

/// Parser for DLT service/control message payloads
///
/// Service messages have a simple structure:
/// - Service ID (4 bytes, little-endian u32)
/// - Parameters (variable length, service-specific)
///
/// # Example
///
/// ```no_run
/// use dlt_protocol::r19_11::*;
///
/// let payload = &[0x00, 0x00, 0x00, 0x01, 0x41, 0x50, 0x50, 0x31, /* ... */];
/// let parser = DltServiceParser::new(payload);
/// 
/// match parser.parse_service_id() {
///     Ok(ServiceId::SetLogLevel) => {
///         let (app_id, ctx_id, log_level) = parser.parse_set_log_level_request().unwrap();
///         println!("SetLogLevel: {:?} {:?} level={}", app_id, ctx_id, log_level);
///     }
///     _ => {}
/// }
/// ```
pub struct DltServiceParser<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> DltServiceParser<'a> {
    /// Create a new service message parser from payload data
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    /// Parse the service ID from the payload
    ///
    /// Service ID is always the first 4 bytes (little-endian u32)
    pub fn parse_service_id(&self) -> Result<ServiceId, DltError> {
        if self.data.len() < 4 {
            return Err(DltError::BufferTooSmall);
        }

        let service_id_value = u32::from_le_bytes([
            self.data[0],
            self.data[1],
            self.data[2],
            self.data[3],
        ]);

        ServiceId::from_u32(service_id_value).ok_or(DltError::InvalidParameter)
    }

    /// Get the raw service ID as u32
    pub fn parse_service_id_raw(&self) -> Result<u32, DltError> {
        if self.data.len() < 4 {
            return Err(DltError::BufferTooSmall);
        }

        Ok(u32::from_le_bytes([
            self.data[0],
            self.data[1],
            self.data[2],
            self.data[3],
        ]))
    }

    /// Get the parameter data (everything after service ID)
    pub fn get_parameters(&self) -> &[u8] {
        if self.data.len() <= 4 {
            &[]
        } else {
            &self.data[4..]
        }
    }

    /// Get the full payload data
    pub fn get_payload(&self) -> &[u8] {
        self.data
    }

    // ========================================
    // Service-Specific Request Parsers
    // ========================================

    /// Parse SetLogLevel request (0x01)
    ///
    /// Returns: (app_id, ctx_id, log_level)
    pub fn parse_set_log_level_request(&self) -> Result<([u8; 4], [u8; 4], i8), DltError> {
        // Expected: 4 (service ID) + 4 (app) + 4 (ctx) + 1 (level) + 4 (reserved) = 17 bytes
        if self.data.len() < 17 {
            return Err(DltError::BufferTooSmall);
        }

        let mut app_id = [0u8; 4];
        app_id.copy_from_slice(&self.data[4..8]);

        let mut ctx_id = [0u8; 4];
        ctx_id.copy_from_slice(&self.data[8..12]);

        let log_level = self.data[12] as i8;

        Ok((app_id, ctx_id, log_level))
    }

    /// Parse SetTraceStatus request (0x02)
    ///
    /// Returns: (app_id, ctx_id, trace_status)
    pub fn parse_set_trace_status_request(&self) -> Result<([u8; 4], [u8; 4], i8), DltError> {
        // Expected: 4 (service ID) + 4 (app) + 4 (ctx) + 1 (status) + 4 (reserved) = 17 bytes
        if self.data.len() < 17 {
            return Err(DltError::BufferTooSmall);
        }

        let mut app_id = [0u8; 4];
        app_id.copy_from_slice(&self.data[4..8]);

        let mut ctx_id = [0u8; 4];
        ctx_id.copy_from_slice(&self.data[8..12]);

        let trace_status = self.data[12] as i8;

        Ok((app_id, ctx_id, trace_status))
    }

    /// Parse GetLogInfo request (0x03)
    ///
    /// Returns: (options, app_id, ctx_id)
    pub fn parse_get_log_info_request(&self) -> Result<(u8, [u8; 4], [u8; 4]), DltError> {
        // Expected: 4 (service ID) + 1 (options) + 4 (app) + 4 (ctx) + 4 (reserved) = 17 bytes
        if self.data.len() < 17 {
            return Err(DltError::BufferTooSmall);
        }

        let options = self.data[4];

        let mut app_id = [0u8; 4];
        app_id.copy_from_slice(&self.data[5..9]);

        let mut ctx_id = [0u8; 4];
        ctx_id.copy_from_slice(&self.data[9..13]);

        Ok((options, app_id, ctx_id))
    }

    /// Parse SetMessageFiltering request (0x0A)
    ///
    /// Returns: filtering_enabled
    pub fn parse_set_message_filtering_request(&self) -> Result<bool, DltError> {
        // Expected: 4 (service ID) + 1 (status) = 5 bytes
        if self.data.len() < 5 {
            return Err(DltError::BufferTooSmall);
        }

        Ok(self.data[4] != 0)
    }

    /// Parse SetDefaultLogLevel request (0x11)
    ///
    /// Returns: log_level
    pub fn parse_set_default_log_level_request(&self) -> Result<i8, DltError> {
        // Expected: 4 (service ID) + 1 (level) + 4 (reserved) = 9 bytes
        if self.data.len() < 9 {
            return Err(DltError::BufferTooSmall);
        }

        Ok(self.data[4] as i8)
    }

    /// Parse GetTraceStatus request (0x1F)
    ///
    /// Returns: (app_id, ctx_id)
    pub fn parse_get_trace_status_request(&self) -> Result<([u8; 4], [u8; 4]), DltError> {
        // Expected: 4 (service ID) + 4 (app) + 4 (ctx) = 12 bytes
        if self.data.len() < 12 {
            return Err(DltError::BufferTooSmall);
        }

        let mut app_id = [0u8; 4];
        app_id.copy_from_slice(&self.data[4..8]);

        let mut ctx_id = [0u8; 4];
        ctx_id.copy_from_slice(&self.data[8..12]);

        Ok((app_id, ctx_id))
    }

    // ========================================
    // Service-Specific Response Parsers
    // ========================================

    /// Parse standard status response (most services)
    ///
    /// Returns: status
    pub fn parse_status_response(&self) -> Result<ServiceStatus, DltError> {
        // Expected: 4 (service ID) + 1 (status) = 5 bytes minimum
        if self.data.len() < 5 {
            return Err(DltError::BufferTooSmall);
        }

        ServiceStatus::from_u8(self.data[4]).ok_or(DltError::InvalidParameter)
    }

    /// Parse GetDefaultLogLevel response (0x04)
    ///
    /// Returns: (status, log_level)
    pub fn parse_get_default_log_level_response(&self) -> Result<(ServiceStatus, u8), DltError> {
        // Expected: 4 (service ID) + 1 (status) + 1 (level) = 6 bytes
        if self.data.len() < 6 {
            return Err(DltError::BufferTooSmall);
        }

        let status = ServiceStatus::from_u8(self.data[4]).ok_or(DltError::InvalidParameter)?;
        let log_level = self.data[5];

        Ok((status, log_level))
    }

    /// Parse GetDefaultTraceStatus response (0x15)
    ///
    /// Returns: (status, trace_status)
    pub fn parse_get_default_trace_status_response(&self) -> Result<(ServiceStatus, u8), DltError> {
        // Expected: 4 (service ID) + 1 (status) + 1 (trace_status) = 6 bytes
        if self.data.len() < 6 {
            return Err(DltError::BufferTooSmall);
        }

        let status = ServiceStatus::from_u8(self.data[4]).ok_or(DltError::InvalidParameter)?;
        let trace_status = self.data[5];

        Ok((status, trace_status))
    }

    /// Parse GetSoftwareVersion response (0x13)
    ///
    /// Returns: (status, sw_version_string)
    /// Note: The length field includes the null terminator, but the returned slice excludes it
    pub fn parse_get_software_version_response(&self) -> Result<(ServiceStatus, &[u8]), DltError> {
        // Expected: 4 (service ID) + 1 (status) + 4 (length) + N (version with null) bytes
        if self.data.len() < 9 {
            return Err(DltError::BufferTooSmall);
        }

        let status = ServiceStatus::from_u8(self.data[4]).ok_or(DltError::InvalidParameter)?;
        
        let length = u32::from_le_bytes([
            self.data[5],
            self.data[6],
            self.data[7],
            self.data[8],
        ]) as usize;

        if self.data.len() < 9 + length {
            return Err(DltError::BufferTooSmall);
        }

        // The length includes null terminator, so actual string is length-1
        let sw_version = if length > 0 && self.data[9 + length - 1] == 0 {
            // String is null-terminated, exclude the null byte
            &self.data[9..9 + length - 1]
        } else {
            // No null terminator found, return full string
            &self.data[9..9 + length]
        };

        Ok((status, sw_version))
    }

    /// Parse GetLogInfo response (0x03)
    ///
    /// Returns: (status, option, payload_data)
    /// The payload_data contains the LogInfo structure that can be parsed with LogInfoResponseParser
    pub fn parse_get_log_info_response(&self) -> Result<(ServiceStatus, &[u8]), DltError> {
        // Expected: 4 (service ID) + 1 (status) + variable (log info data) + 4 (reserved)
        if self.data.len() < 9 {
            return Err(DltError::BufferTooSmall);
        }

        let status = ServiceStatus::from_u8(self.data[4]).ok_or(DltError::InvalidParameter)?;
        
        // The rest is log info data (excluding the last 4 reserved bytes if present)
        let payload_start = 5;
        let payload_end = if self.data.len() >= 9 {
            self.data.len() - 4 // Exclude 4-byte reserved field at end
        } else {
            self.data.len()
        };
        
        let log_info_data = &self.data[payload_start..payload_end];
        
        Ok((status, log_info_data))
    }

    /// Parse GetLogChannelNames response (0x17)
    ///
    /// Returns: (status, channel_names)
    pub fn parse_get_log_channel_names_response(&self) -> Result<(ServiceStatus, &[u8]), DltError> {
        // Expected: 4 (service ID) + 1 (status) + 1 (count) + N×4 (channel names)
        if self.data.len() < 6 {
            return Err(DltError::BufferTooSmall);
        }

        let status = ServiceStatus::from_u8(self.data[4]).ok_or(DltError::InvalidParameter)?;
        let count = self.data[5] as usize;

        if self.data.len() < 6 + (count * 4) {
            return Err(DltError::BufferTooSmall);
        }

        let channel_names = &self.data[6..6 + (count * 4)];

        Ok((status, channel_names))
    }

    /// Parse GetTraceStatus response (0x1F)
    ///
    /// Returns: (status, trace_status)
    pub fn parse_get_trace_status_response(&self) -> Result<(ServiceStatus, u8), DltError> {
        // Expected: 4 (service ID) + 1 (status) + 1 (trace_status) = 6 bytes
        if self.data.len() < 6 {
            return Err(DltError::BufferTooSmall);
        }

        let status = ServiceStatus::from_u8(self.data[4]).ok_or(DltError::InvalidParameter)?;
        let trace_status = self.data[5];

        Ok((status, trace_status))
    }

    /// Parse BufferOverflowNotification response (0x23)
    ///
    /// Returns: (status, overflow_counter)
    pub fn parse_buffer_overflow_notification(&self) -> Result<(ServiceStatus, u32), DltError> {
        // Expected: 4 (service ID) + 1 (status) + 4 (counter) = 9 bytes
        if self.data.len() < 9 {
            return Err(DltError::BufferTooSmall);
        }

        let status = ServiceStatus::from_u8(self.data[4]).ok_or(DltError::InvalidParameter)?;
        
        let overflow_counter = u32::from_le_bytes([
            self.data[5],
            self.data[6],
            self.data[7],
            self.data[8],
        ]);

        Ok((status, overflow_counter))
    }

    // ========================================
    // Advanced Parsing with Position Tracking
    // ========================================

    /// Reset parser position to beginning
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Read a single byte at current position and advance
    pub fn read_u8(&mut self) -> Result<u8, DltError> {
        if self.position >= self.data.len() {
            return Err(DltError::BufferTooSmall);
        }
        let value = self.data[self.position];
        self.position += 1;
        Ok(value)
    }

    /// Read a u16 (little-endian) at current position and advance
    pub fn read_u16_le(&mut self) -> Result<u16, DltError> {
        if self.position + 2 > self.data.len() {
            return Err(DltError::BufferTooSmall);
        }
        let value = u16::from_le_bytes([
            self.data[self.position],
            self.data[self.position + 1],
        ]);
        self.position += 2;
        Ok(value)
    }

    /// Read a u32 (little-endian) at current position and advance
    pub fn read_u32_le(&mut self) -> Result<u32, DltError> {
        if self.position + 4 > self.data.len() {
            return Err(DltError::BufferTooSmall);
        }
        let value = u32::from_le_bytes([
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
        ]);
        self.position += 4;
        Ok(value)
    }

    /// Read N bytes at current position and advance
    pub fn read_bytes(&mut self, count: usize) -> Result<&[u8], DltError> {
        if self.position + count > self.data.len() {
            return Err(DltError::BufferTooSmall);
        }
        let bytes = &self.data[self.position..self.position + count];
        self.position += count;
        Ok(bytes)
    }

    /// Read a 4-byte ID at current position and advance
    pub fn read_id(&mut self) -> Result<[u8; 4], DltError> {
        if self.position + 4 > self.data.len() {
            return Err(DltError::BufferTooSmall);
        }
        let mut id = [0u8; 4];
        id.copy_from_slice(&self.data[self.position..self.position + 4]);
        self.position += 4;
        Ok(id)
    }

    /// Get current position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get remaining bytes from current position
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }

    /// Get remaining bytes as slice
    pub fn remaining_bytes(&self) -> &[u8] {
        &self.data[self.position..]
    }
}

// ========================================
// GetLogInfo Response Parser
// ========================================

/// Parser for GetLogInfo response payload
///
/// Parses the complex nested structure of GetLogInfo responses:
/// - appIdCount (2 bytes)
/// - For each app:
///   - app_id (4 bytes)
///   - contextIdCount (2 bytes)
///   - For each context:
///     - context_id (4 bytes)
///     - log_level (1 byte)
///     - trace_status (1 byte)
///     - [if option 7:] description_len (2 bytes) + description (N bytes)
///   - [if option 7:] app_description_len (2 bytes) + description (N bytes)
pub struct LogInfoResponseParser<'a> {
    data: &'a [u8],
    position: usize,
    with_descriptions: bool,
}

impl<'a> LogInfoResponseParser<'a> {
    /// Create a new LogInfo response parser
    ///
    /// # Arguments
    /// * `data` - The log info payload (after service ID and status)
    /// * `with_descriptions` - true for option 7, false for option 6
    pub fn new(data: &'a [u8], with_descriptions: bool) -> Self {
        Self {
            data,
            position: 0,
            with_descriptions,
        }
    }

    /// Get the number of applications
    pub fn read_app_count(&mut self) -> Result<u16, DltError> {
        if self.position + 2 > self.data.len() {
            return Err(DltError::BufferTooSmall);
        }
        let count = u16::from_le_bytes([self.data[self.position], self.data[self.position + 1]]);
        self.position += 2;
        Ok(count)
    }

    /// Read application ID
    pub fn read_app_id(&mut self) -> Result<[u8; 4], DltError> {
        if self.position + 4 > self.data.len() {
            return Err(DltError::BufferTooSmall);
        }
        let mut app_id = [0u8; 4];
        app_id.copy_from_slice(&self.data[self.position..self.position + 4]);
        self.position += 4;
        Ok(app_id)
    }

    /// Read context count for current application
    pub fn read_context_count(&mut self) -> Result<u16, DltError> {
        if self.position + 2 > self.data.len() {
            return Err(DltError::BufferTooSmall);
        }
        let count = u16::from_le_bytes([self.data[self.position], self.data[self.position + 1]]);
        self.position += 2;
        Ok(count)
    }

    /// Read context information (ID, log level, trace status)
    pub fn read_context_info(&mut self) -> Result<([u8; 4], u8, u8), DltError> {
        if self.position + 6 > self.data.len() {
            return Err(DltError::BufferTooSmall);
        }
        
        let mut context_id = [0u8; 4];
        context_id.copy_from_slice(&self.data[self.position..self.position + 4]);
        let log_level = self.data[self.position + 4];
        let trace_status = self.data[self.position + 5];
        self.position += 6;
        
        Ok((context_id, log_level, trace_status))
    }

    /// Read description (if with_descriptions is true)
    /// Returns the description bytes without length prefix
    pub fn read_description(&mut self) -> Result<&'a [u8], DltError> {
        if !self.with_descriptions {
            return Ok(&[]);
        }

        if self.position + 2 > self.data.len() {
            return Err(DltError::BufferTooSmall);
        }

        let len = u16::from_le_bytes([self.data[self.position], self.data[self.position + 1]]) as usize;
        self.position += 2;

        if self.position + len > self.data.len() {
            return Err(DltError::BufferTooSmall);
        }

        let desc = &self.data[self.position..self.position + len];
        self.position += len;
        Ok(desc)
    }

    /// Get current position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Check if there's more data to read
    pub fn has_remaining(&self) -> bool {
        self.position < self.data.len()
    }
}

// ========================================
// Helper Functions
// ========================================

/// Check if an ID is all zeros (wildcard)
pub fn is_wildcard_id(id: &[u8; 4]) -> bool {
    id == &[0, 0, 0, 0]
}

/// Convert DLT ID to string, handling null bytes
pub fn id_to_string(id: &[u8; 4]) -> core::result::Result<&str, core::str::Utf8Error> {
    // Find first null byte
    let end = id.iter().position(|&b| b == 0).unwrap_or(4);
    core::str::from_utf8(&id[..end])
}
