//! # DLT Protocol R19.11 Header Definitions
//!
//! This module implements the AUTOSAR DLT (Diagnostic Log and Trace) protocol 
//! header structures and parsing logic according to specification release 19.11.
//!
//! ## DLT Message Structure
//!
//! A complete DLT message consists of:
//! 1. **Serial Header** (4 bytes, optional): "DLS\x01"
//! 2. **Standard Header** (4 bytes, required): HTYP, Message Counter, Length
//! 3. **Standard Header Extra** (0-12 bytes, conditional): ECU ID, Session ID, Timestamp
//! 4. **Extended Header** (10 bytes, optional): MSIN, NOAR, App ID, Context ID
//! 5. **Payload** (variable length): Message data
//!
//! ## MSIN Byte Structure (Extended Header)
//!
//! The MSIN byte encodes message type information (per DLT R19.11 spec):
//! ```text
//! Bit 0:     VERB - Verbose mode flag (0=non-verbose, 1=verbose)
//! Bits 1-3:  MSTP - Message Type (3 bits, values 0-7)
//!            0 = Log, 1 = App Trace, 2 = NW Trace, 3 = Control
//! Bits 4-7:  MTIN - Message Type Info (4 bits, values 0-15)
//!            For Log: 1=Fatal, 2=Error, 3=Warn, 4=Info, 5=Debug, 6=Verbose
//! ```

// ========================================
// Size Constants
// ========================================

/// DLT ID field size (ECU ID, App ID, Context ID)
pub const DLT_ID_SIZE: usize = 4;

/// Storage header size (not used in runtime messages)
pub const DLT_STORAGE_HEADER_SIZE: usize = 16;

/// Standard header size (HTYP + MCNT + LEN)
pub const DLT_STANDARD_HEADER_SIZE: usize = 4;

/// Extended header size (MSIN + NOAR + APID + CTID)
pub const DLT_EXTENDED_HEADER_SIZE: usize = 10;

/// Standard header extra size with all optional fields
pub const DLT_STANDARD_HEADER_EXTRA_SIZE: usize = 12;

/// Standard header extra size without session ID
pub const DLT_STANDARD_HEADER_EXTRA_NOSESSIONID_SIZE: usize = 8;

/// Payload header size for typed arguments
pub const DLT_PAYLOAD_HEADER_SIZE: usize = 6;

// ========================================
// HTYP Byte Bit Masks (Standard Header)
// ========================================

/// Bit 0: Use Extended Header
pub const UEH_MASK: u8 = 0x01;

/// Bit 1: Most Significant Byte First (Big Endian)
pub const MSBF_MASK: u8 = 0x02;

/// Bit 2: With ECU ID
pub const WEID_MASK: u8 = 0x04;

/// Bit 3: With Session ID
pub const WSID_MASK: u8 = 0x08;

/// Bit 4: With Timestamp
pub const WTMS_MASK: u8 = 0x10;

/// Bits 5-7: Version Number
pub const VERS_MASK: u8 = 0xE0;

/// DLT Protocol Version 1.0
pub const DLT_VERSION: u8 = 0x10;

// ========================================
// Serial Header Constants
// ========================================

/// Serial header size: 4 bytes
pub const DLT_SERIAL_HEADER_SIZE: usize = 4;

/// Serial header pattern: "DLS" + 0x01
pub const DLT_SERIAL_HEADER_ARRAY: [u8; DLT_SERIAL_HEADER_SIZE] = [0x44, 0x4C, 0x53, 0x01];

// ========================================
// MSIN Byte Bit Positions (Extended Header)
// ========================================

/// Bit 0: Verbose mode flag
const MSIN_VERBOSE_BIT: u8 = 0;

/// Bits 1-3: Message Type (MSTP) - 3 bits
const MSIN_MSTP_SHIFT: u8 = 1;
const MSIN_MSTP_MASK: u8 = 0x07; // 3 bits: 0b0111

/// Bits 4-7: Message Type Info (MTIN) - 4 bits
const MSIN_MTIN_SHIFT: u8 = 4;
const MSIN_MTIN_MASK: u8 = 0x0F; // 4 bits: 0b1111

// ========================================
// Header Type Structures
// ========================================

/// HTYP byte decoded structure (Standard Header byte 0)
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DltHTYP {
    /// Use Extended Header
    pub UEH: bool,
    /// Most Significant Byte First (Big Endian)
    pub MSBF: bool,
    /// With ECU ID
    pub WEID: bool,
    /// With Session ID
    pub WSID: bool,
    /// With Timestamp  
    pub WTMS: bool,
    /// Version number (should be 1 for DLT 1.0)
    pub VERS: u8,
}

/// DLT Standard Header (4 bytes, always present)
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct DltStandardHeader {
    /// Header Type byte (HTYP)
    pub htyp: u8,
    /// Message Counter (wraps at 255)
    pub mcnt: u8,
    /// Length of message from standard header to end of payload (excludes serial header)
    pub len: u16,
}

/// DLT Standard Header Extra Fields (0-12 bytes, conditional)
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct DltStandardHeaderExtra {
    /// ECU ID (4 bytes, present if WEID bit set)
    pub ecu: [u8; DLT_ID_SIZE],
    /// Session ID (4 bytes, present if WSID bit set)
    pub seid: u32,
    /// Timestamp in 0.1ms units (4 bytes, present if WTMS bit set)
    pub tmsp: u32,
}

/// DLT Extended Header (10 bytes, optional)
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct DltExtendedHeader {
    /// Message Info byte (VERB, MSTP, MTIN)
    pub msin: u8,
    /// Number of Arguments
    pub noar: u8,
    /// Application ID (4 bytes)
    pub apid: [u8; DLT_ID_SIZE],
    /// Context ID (4 bytes)
    pub ctid: [u8; DLT_ID_SIZE],
}

// ========================================
// Message Type Enumerations
// ========================================

/// DLT Log Level (legacy, consider using MtinTypeDltLog)
#[derive(Debug, Clone, Copy)]
pub enum DltLogLevel {
    Fatal,
    Error,
    Warn,
    Info,
    Debug,
    Verbose,
    Reserved(u8),
    Invalid(u8),
}

/// Message Type (MSTP) - 3 bits, values 0-7
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MstpType {
    /// Type 0: Log message
    DltTypeLog,
    /// Type 1: Application trace
    DltTypeAppTrace,
    /// Type 2: Network trace
    DltTypeNwTrace,
    /// Type 3: Control message
    DltTypeControl,
    /// Types 4-7: Reserved for future use
    Reserved(u8),
    /// Invalid type (> 7)
    Invalid(u8),
}

impl MstpType {
    /// Parse MSTP value from 3-bit field (values 0-7)
    pub fn parse(value: u8) -> MstpType {
        match value {
            0x0 => MstpType::DltTypeLog,
            0x1 => MstpType::DltTypeAppTrace,
            0x2 => MstpType::DltTypeNwTrace,
            0x3 => MstpType::DltTypeControl,
            0x4..=0x7 => MstpType::Reserved(value),
            _ => MstpType::Invalid(value),
        }
    }

    /// Convert to 3-bit value
    pub fn to_bits(&self) -> u8 {
        match self {
            MstpType::DltTypeLog => 0x0,
            MstpType::DltTypeAppTrace => 0x1,
            MstpType::DltTypeNwTrace => 0x2,
            MstpType::DltTypeControl => 0x3,
            MstpType::Reserved(v) => *v,
            MstpType::Invalid(v) => *v,
        }
    }
}

/// Message Type Info (MTIN) - type-specific enumeration
#[derive(Debug)]
pub enum Mtin {
    Log(MtinTypeDltLog),
    AppTrace(MtinTypeDltAppTrace),
    NwTrace(MtinTypeDltNwTrace),
    Control(MtinTypeDltControl),
    Invalid,
}

/// Message Type Info for Log messages (MTIN when MSTP=0)
#[derive(Debug, Clone, Copy)]
pub enum MtinTypeDltLog {
    /// Level 1: Fatal error
    DltLogFatal,
    /// Level 2: Error
    DltLogError,
    /// Level 3: Warning
    DltLogWarn,
    /// Level 4: Info
    DltLogInfo,
    /// Level 5: Debug
    DltLogDebug,
    /// Level 6: Verbose
    DltLogVerbose,
    /// Levels 7-15: Reserved
    Reserved(u8),
    /// Invalid level (0 or implementation-specific)
    Invalid(u8),
}

impl MtinTypeDltLog {
    /// Parse log level from 4-bit MTIN field
    pub fn parse(value: u8) -> MtinTypeDltLog {
        match value {
            0x1 => MtinTypeDltLog::DltLogFatal,
            0x2 => MtinTypeDltLog::DltLogError,
            0x3 => MtinTypeDltLog::DltLogWarn,
            0x4 => MtinTypeDltLog::DltLogInfo,
            0x5 => MtinTypeDltLog::DltLogDebug,
            0x6 => MtinTypeDltLog::DltLogVerbose,
            _ => MtinTypeDltLog::Reserved(value),
        }
    }

    /// Convert to 4-bit value
    pub fn to_bits(&self) -> u8 {
        match self {
            MtinTypeDltLog::DltLogFatal => 0x1,
            MtinTypeDltLog::DltLogError => 0x2,
            MtinTypeDltLog::DltLogWarn => 0x3,
            MtinTypeDltLog::DltLogInfo => 0x4,
            MtinTypeDltLog::DltLogDebug => 0x5,
            MtinTypeDltLog::DltLogVerbose => 0x6,
            MtinTypeDltLog::Reserved(v) => *v,
            MtinTypeDltLog::Invalid(v) => *v,
        }
    }
}

/// Message Type Info for Application Trace (MTIN when MSTP=1)
#[derive(Debug)]
pub enum MtinTypeDltAppTrace {
    DltTraceVariable,
    Reserved(u8),
    Invalid(u8),
}

impl MtinTypeDltAppTrace {
    pub fn parse(value: u8) -> MtinTypeDltAppTrace {
        MtinTypeDltAppTrace::Invalid(value)
    }
}

/// Message Type Info for Network Trace (MTIN when MSTP=2)
#[derive(Debug)]
pub enum MtinTypeDltNwTrace {
    DltTraceVariable,
    Reserved(u8),
    Invalid(u8),
}

impl MtinTypeDltNwTrace {
    pub fn parse(value: u8) -> MtinTypeDltNwTrace {
        MtinTypeDltNwTrace::Invalid(value)
    }
}

/// Message Type Info for Control messages (MTIN when MSTP=3)
#[derive(Debug)]
pub enum MtinTypeDltControl {
    /// Control Request
    DltControlRequest,
    /// Control Response
    DltControlResponse,
    /// Reserved values
    Reserved(u8),
    /// Invalid value
    Invalid(u8),
}

impl MtinTypeDltControl {
    pub fn parse(value: u8) -> MtinTypeDltControl {
        match value {
            0x1 => MtinTypeDltControl::DltControlRequest,
            0x2 => MtinTypeDltControl::DltControlResponse,
            0x3..=0x7 => MtinTypeDltControl::Reserved(value),
            _ => MtinTypeDltControl::Invalid(value),
        }
    }
}

impl Mtin {
    /// Parse MTIN based on MSTP type
    pub fn parse(mstp: &MstpType, value: u8) -> Mtin {
        match mstp {
            MstpType::DltTypeLog => Mtin::Log(MtinTypeDltLog::parse(value)),
            MstpType::DltTypeAppTrace => Mtin::AppTrace(MtinTypeDltAppTrace::parse(value)),
            MstpType::DltTypeNwTrace => Mtin::NwTrace(MtinTypeDltNwTrace::parse(value)),
            MstpType::DltTypeControl => Mtin::Control(MtinTypeDltControl::parse(value)),
            _ => Mtin::Invalid,
        }
    }
}

// ========================================
// Parser Error Types
// ========================================

/// Errors that can occur during DLT message parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DltHeaderError {
    /// Buffer is too small for the expected data
    BufferTooSmall,
    /// DLT version is not 1
    InvalidVersion,
    /// Serial header pattern doesn't match "DLS\x01"
    InvalidSerialHeader,
    /// Invalid header type flags
    InvalidHeaderType,
}

// ========================================
// Parsed Message Structure
// ========================================

/// Complete parsed DLT message with all header information and payload
#[derive(Debug, Clone, Copy)]
pub struct DltMessage<'a> {
    /// Whether the message included a serial header
    pub has_serial_header: bool,
    /// Standard header (always present)
    pub standard_header: DltStandardHeader,
    /// Decoded header type flags
    pub header_type: DltHTYP,
    /// ECU ID (present if WEID flag set)
    pub ecu_id: Option<[u8; DLT_ID_SIZE]>,
    /// Session ID (present if WSID flag set)
    pub session_id: Option<u32>,
    /// Timestamp in 0.1ms units (present if WTMS flag set)
    pub timestamp: Option<u32>,
    /// Extended header (present if UEH flag set)
    pub extended_header: Option<DltExtendedHeader>,
    /// Message payload (raw bytes)
    pub payload: &'a [u8],
}

// ========================================
// DLT Header Parser
// ========================================

/// Parser for DLT protocol messages
///
/// # Example
/// ```no_run
/// use dlt_protocol::r19_11::*;
///
/// let data: &[u8] = &[/* DLT packet bytes */];
/// let mut parser = DltHeaderParser::new(data);
/// 
/// match parser.parse_message() {
///     Ok(message) => {
///         println!("ECU: {:?}", message.ecu_id);
///         if let Some(ext) = message.extended_header {
///             println!("App ID: {:?}", ext.apid);
///             println!("Log level: {:?}", ext.log_level());
///         }
///     }
///     Err(e) => eprintln!("Parse error: {:?}", e),
/// }
/// ```
pub struct DltHeaderParser<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> DltHeaderParser<'a> {
    /// Create a new parser from raw packet data
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    /// Parse a complete DLT message including all headers and payload
    ///
    /// This method parses:
    /// 1. Optional serial header ("DLS\x01")
    /// 2. Standard header (HTYP, MCNT, LEN)
    /// 3. Optional extra fields (ECU ID, Session ID, Timestamp)
    /// 4. Optional extended header (MSIN, NOAR, APID, CTID)
    /// 5. Payload (remaining bytes based on LEN field)
    ///
    /// # Returns
    /// - `Ok(DltMessage)`: Successfully parsed message
    /// - `Err(DltHeaderError)`: Parsing failed (buffer too small, invalid version, etc.)
    pub fn parse_message(&mut self) -> Result<DltMessage<'a>, DltHeaderError> {
        let start_position = self.position;
        
        // Check for optional serial header
        let has_serial = self.check_serial_header();
        if has_serial {
            self.skip_serial_header()?;
        }

        // Parse standard header (required)
        let standard_header = self.parse_standard_header()?;
        let header_type = Self::decode_htyp(standard_header.htyp);

        // Parse standard header extra fields (optional)
        let (ecu_id, session_id, timestamp) = self.parse_standard_header_extra(&header_type)?;

        // Parse extended header (optional)
        let extended_header = if header_type.UEH {
            Some(self.parse_extended_header()?)
        } else {
            None
        };

        // Calculate payload offset and extract payload
        // The 'len' field in standard header includes everything from standard header to end of payload
        // It does NOT include the serial header
        let total_len = standard_header.len as usize;
        
        // Calculate how many header bytes we've consumed (excluding serial header)
        let header_bytes_consumed = if has_serial {
            self.position - start_position - DLT_SERIAL_HEADER_SIZE
        } else {
            self.position - start_position
        };
        
        // Payload length = total_len - header_bytes_consumed
        let payload_len = total_len.saturating_sub(header_bytes_consumed);
        let payload_start = self.position;
        let payload_end = payload_start + payload_len;
        
        if payload_end > self.data.len() {
            return Err(DltHeaderError::BufferTooSmall);
        }
        
        let payload = &self.data[payload_start..payload_end];
        self.position = payload_end;
        
        Ok(DltMessage {
            has_serial_header: has_serial,
            standard_header,
            header_type,
            ecu_id,
            session_id,
            timestamp,
            extended_header,
            payload,
        })
    }

    /// Check if the buffer starts with a serial header
    fn check_serial_header(&self) -> bool {
        if self.position + DLT_SERIAL_HEADER_SIZE > self.data.len() {
            return false;
        }
        &self.data[self.position..self.position + DLT_SERIAL_HEADER_SIZE] == &DLT_SERIAL_HEADER_ARRAY
    }

    /// Skip the serial header
    fn skip_serial_header(&mut self) -> Result<(), DltHeaderError> {
        if self.position + DLT_SERIAL_HEADER_SIZE > self.data.len() {
            return Err(DltHeaderError::BufferTooSmall);
        }
        self.position += DLT_SERIAL_HEADER_SIZE;
        Ok(())
    }

    /// Parse the standard header (4 bytes)
    fn parse_standard_header(&mut self) -> Result<DltStandardHeader, DltHeaderError> {
        if self.position + DLT_STANDARD_HEADER_SIZE > self.data.len() {
            return Err(DltHeaderError::BufferTooSmall);
        }

        let htyp = self.data[self.position];
        let mcnt = self.data[self.position + 1];
        
        // Check version
        let version = (htyp & VERS_MASK) >> 5;
        if version != 1 {
            return Err(DltHeaderError::InvalidVersion);
        }

        // Per DLT spec PRS_Dlt_00091: Standard header uses network byte order (big-endian)
        // regardless of MSBF flag. The MSBF flag only affects payload and extended header.
        let len = u16::from_be_bytes([self.data[self.position + 2], self.data[self.position + 3]]);

        self.position += DLT_STANDARD_HEADER_SIZE;

        Ok(DltStandardHeader { htyp, mcnt, len })
    }

    /// Decode HTYP byte into structured format
    fn decode_htyp(htyp: u8) -> DltHTYP {
        DltHTYP {
            UEH: (htyp & UEH_MASK) != 0,
            MSBF: (htyp & MSBF_MASK) != 0,
            WEID: (htyp & WEID_MASK) != 0,
            WSID: (htyp & WSID_MASK) != 0,
            WTMS: (htyp & WTMS_MASK) != 0,
            VERS: (htyp & VERS_MASK) >> 5,
        }
    }

    /// Parse standard header extra fields (ECU ID, Session ID, Timestamp)
    fn parse_standard_header_extra(
        &mut self,
        header_type: &DltHTYP,
    ) -> Result<(Option<[u8; DLT_ID_SIZE]>, Option<u32>, Option<u32>), DltHeaderError> {
        let mut ecu_id = None;
        let mut session_id = None;
        let mut timestamp = None;

        // ECU ID (4 bytes)
        if header_type.WEID {
            if self.position + DLT_ID_SIZE > self.data.len() {
                return Err(DltHeaderError::BufferTooSmall);
            }
            let mut ecu = [0u8; DLT_ID_SIZE];
            ecu.copy_from_slice(&self.data[self.position..self.position + DLT_ID_SIZE]);
            ecu_id = Some(ecu);
            self.position += DLT_ID_SIZE;
        }

        // Session ID (4 bytes)
        // Always in big-endian per DLT spec
        if header_type.WSID {
            if self.position + 4 > self.data.len() {
                return Err(DltHeaderError::BufferTooSmall);
            }
            let seid = {
                u32::from_be_bytes([
                    self.data[self.position],
                    self.data[self.position + 1],
                    self.data[self.position + 2],
                    self.data[self.position + 3],
                ])
            };
            session_id = Some(seid);
            self.position += 4;
        }

        // Timestamp (4 bytes)
        // Always in big-endian per DLT spec
        if header_type.WTMS {
            if self.position + 4 > self.data.len() {
                return Err(DltHeaderError::BufferTooSmall);
            }
            let tmsp = {
                u32::from_be_bytes([
                    self.data[self.position],
                    self.data[self.position + 1],
                    self.data[self.position + 2],
                    self.data[self.position + 3],
                ])
            };
            timestamp = Some(tmsp);
            self.position += 4;
        }

        Ok((ecu_id, session_id, timestamp))
    }

    /// Parse extended header (10 bytes)
    fn parse_extended_header(&mut self) -> Result<DltExtendedHeader, DltHeaderError> {
        if self.position + DLT_EXTENDED_HEADER_SIZE > self.data.len() {
            return Err(DltHeaderError::BufferTooSmall);
        }

        let msin = self.data[self.position];
        let noar = self.data[self.position + 1];

        let mut apid = [0u8; DLT_ID_SIZE];
        apid.copy_from_slice(&self.data[self.position + 2..self.position + 6]);

        let mut ctid = [0u8; DLT_ID_SIZE];
        ctid.copy_from_slice(&self.data[self.position + 6..self.position + 10]);

        self.position += DLT_EXTENDED_HEADER_SIZE;

        Ok(DltExtendedHeader {
            msin,
            noar,
            apid,
            ctid,
        })
    }

    /// Get current parsing position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get remaining unparsed data
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }
}

// ========================================
// MSIN Byte Helper Functions
// ========================================

/// Encode MSIN byte from components (per DLT R19.11 spec)
///
/// # Arguments
/// * `verbose` - Verbose mode flag (bit 0)
/// * `mstp` - Message type (bits 1-3, 3-bit value 0-7)
/// * `mtin` - Message type info (bits 4-7, 4-bit value 0-15)
///
/// # Returns
/// Encoded MSIN byte with the following bit layout:
/// ```text
/// Bit 0:     VERB (verbose flag)
/// Bits 1-3:  MSTP (message type, 3 bits)
/// Bits 4-7:  MTIN (message type info, 4 bits)
/// ```
#[inline]
pub fn encode_msin(verbose: bool, mstp: u8, mtin: u8) -> u8 {
    let verbose_bit = if verbose { 0x01 } else { 0x00 };
    let mstp_bits = (mstp & MSIN_MSTP_MASK) << MSIN_MSTP_SHIFT;
    let mtin_bits = (mtin & MSIN_MTIN_MASK) << MSIN_MTIN_SHIFT;
    verbose_bit | mstp_bits | mtin_bits
}

/// Extract verbose flag from MSIN byte (bit 0)
#[inline]
pub fn extract_msin_verbose(msin: u8) -> bool {
    (msin & (1 << MSIN_VERBOSE_BIT)) != 0
}

/// Extract MSTP (message type) from MSIN byte (bits 1-3)
#[inline]
pub fn extract_msin_mstp(msin: u8) -> u8 {
    (msin >> MSIN_MSTP_SHIFT) & MSIN_MSTP_MASK
}

/// Extract MTIN (message type info) from MSIN byte (bits 4-7)
#[inline]
pub fn extract_msin_mtin(msin: u8) -> u8 {
    (msin >> MSIN_MTIN_SHIFT) & MSIN_MTIN_MASK
}

// ========================================
// Extended Header Helper Methods
// ========================================

/// Helper methods for interpreting parsed extended headers
impl DltExtendedHeader {
    /// Check if message is in verbose mode (bit 0 of MSIN)
    pub fn is_verbose(&self) -> bool {
        extract_msin_verbose(self.msin)
    }

    /// Get message type (MSTP) from MSIN byte (bits 1-3)
    ///
    /// Returns the decoded message type: Log, AppTrace, NwTrace, or Control
    pub fn message_type(&self) -> MstpType {
        let mstp_bits = extract_msin_mstp(self.msin);
        MstpType::parse(mstp_bits)
    }

    /// Get raw message type info value (MTIN) from MSIN byte (bits 4-7)
    ///
    /// Returns the 4-bit MTIN value (0-15). Interpretation depends on message type.
    pub fn message_type_info(&self) -> u8 {
        extract_msin_mtin(self.msin)
    }

    /// Get log level if this is a Log message type
    ///
    /// # Returns
    /// - `Some(MtinTypeDltLog)`: If MSTP indicates this is a log message
    /// - `None`: If this is not a log message (AppTrace, NwTrace, Control)
    pub fn log_level(&self) -> Option<MtinTypeDltLog> {
        if matches!(self.message_type(), MstpType::DltTypeLog) {
            Some(MtinTypeDltLog::parse(self.message_type_info()))
        } else {
            None
        }
    }
}

