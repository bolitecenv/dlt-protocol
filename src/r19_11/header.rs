#![no_std]

pub const DLT_ID_SIZE: usize = 4;

pub const DLT_STORAGE_HEADER_SIZE: usize = 16;
pub const DLT_STANDARD_HEADER_SIZE: usize = 4;
pub const DLT_EXTENDED_HEADER_SIZE: usize = 10;
pub const DLT_STANDARD_HEADER_EXTRA_SIZE: usize = 12;
pub const DLT_STANDARD_HEADER_EXTRA_NOSESSIONID_SIZE: usize = 8;
pub const DLT_PAYLOAD_HEADER_SIZE: usize = 6;

pub const UEH_MASK: u8 = 0x01; // Bit 0: Use Extended Header
pub const MSBF_MASK: u8 = 0x02; // Bit 1: Most Significant Byte First
pub const WEID_MASK: u8 = 0x04; // Bit 2: With ECU ID
pub const WSID_MASK: u8 = 0x08; // Bit 3: With Session ID
pub const WTMS_MASK: u8 = 0x10; // Bit 4: With Timestamp
pub const VERS_MASK: u8 = 0xE0; // Bit 5-7: Version Number (11100000)
pub const DLT_VERSION: u8 = 0x10; // Version 1.0 (00010000)

// Serial Header Constants 4 bytes, "DLS" + 0x01 
pub const DLT_SERIAL_HEADER_SIZE: usize = 4;
pub const DLT_SERIAL_HEADER_ARRAY: [u8; DLT_SERIAL_HEADER_SIZE] = [0x44, 0x4C, 0x53, 0x01]; // "DLS" + 0x01

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DltHTYP {
    pub UEH: bool,
    pub MSBF: bool,
    pub WEID: bool,
    pub WSID: bool,
    pub WTMS: bool,
    pub VERS: u8,
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct DltStandardHeader {
    pub htyp: u8,
    pub mcnt: u8,
    pub len: u16,
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct DltStandardHeaderExtra {
    pub ecu: [u8; DLT_ID_SIZE],
    pub seid: u32,
    pub tmsp: u32,
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct DltExtendedHeader {
    pub msin: u8,
    pub noar: u8,
    pub apid: [u8; DLT_ID_SIZE],
    pub ctid: [u8; DLT_ID_SIZE],
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MstpType {
    DltTypeLog,
    DltTypeAppTrace,
    DltTypeNwTrace,
    DltTypeControl,
    Reserved(u8),
    Invalid(u8),
}

impl MstpType {
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

#[derive(Debug)]
pub enum Mtin {
    Log(MtinTypeDltLog),
    AppTrace(MtinTypeDltAppTrace),
    NwTrace(MtinTypeDltNwTrace),
    Control(MtinTypeDltControl),
    Invalid,
}

#[derive(Debug, Clone, Copy)]
pub enum MtinTypeDltLog {
    DltLogFatal,
    DltLogError,
    DltLogWarn,
    DltLogInfo,
    DltLogDebug,
    DltLogVerbose,
    Reserved(u8),
    Invalid(u8),
}

#[derive(Debug)]
pub enum MtinTypeDltAppTrace {
    DltTraceVariable,
    Reserved(u8),
    Invalid(u8),
}

#[derive(Debug)]
pub enum MtinTypeDltNwTrace {
    DltTraceVariable,
    Reserved(u8),
    Invalid(u8),
}

#[derive(Debug)]
pub enum MtinTypeDltControl {
    DltControlRequest,
    DltControlResponse,
    Reserved(u8),
    Invalid(u8),
}

impl MtinTypeDltLog {
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

impl MtinTypeDltAppTrace {
    pub fn parse(value: u8) -> MtinTypeDltAppTrace {
        MtinTypeDltAppTrace::Invalid(value)
    }
}

impl MtinTypeDltNwTrace {
    pub fn parse(value: u8) -> MtinTypeDltNwTrace {
        MtinTypeDltNwTrace::Invalid(value)
    }
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
// DLT Header Parser
// ========================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DltHeaderError {
    BufferTooSmall,
    InvalidVersion,
    InvalidSerialHeader,
    InvalidHeaderType,
}

/// Parsed DLT message with all header information
#[derive(Debug, Clone, Copy)]
pub struct DltMessage<'a> {
    pub has_serial_header: bool,
    pub standard_header: DltStandardHeader,
    pub header_type: DltHTYP,
    pub ecu_id: Option<[u8; DLT_ID_SIZE]>,
    pub session_id: Option<u32>,
    pub timestamp: Option<u32>,
    pub extended_header: Option<DltExtendedHeader>,
    pub payload: &'a [u8],
}

/// DLT Header Parser for parsing incoming packets
pub struct DltHeaderParser<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> DltHeaderParser<'a> {
    /// Create a new header parser from raw packet data
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    /// Parse a complete DLT message including all headers
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
        if header_type.WSID {
            if self.position + 4 > self.data.len() {
                return Err(DltHeaderError::BufferTooSmall);
            }
            let seid = if header_type.MSBF {
                u32::from_be_bytes([
                    self.data[self.position],
                    self.data[self.position + 1],
                    self.data[self.position + 2],
                    self.data[self.position + 3],
                ])
            } else {
                u32::from_le_bytes([
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
        if header_type.WTMS {
            if self.position + 4 > self.data.len() {
                return Err(DltHeaderError::BufferTooSmall);
            }
            let tmsp = if header_type.MSBF {
                u32::from_be_bytes([
                    self.data[self.position],
                    self.data[self.position + 1],
                    self.data[self.position + 2],
                    self.data[self.position + 3],
                ])
            } else {
                u32::from_le_bytes([
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

/// Helper functions for interpreting parsed headers
impl DltExtendedHeader {
    /// Check if message is verbose mode
    pub fn is_verbose(&self) -> bool {
        (self.msin & 0x01) != 0
    }

    /// Get message type (MSTP)
    pub fn message_type(&self) -> MstpType {
        let mstp_bits = (self.msin >> 1) & 0x07;
        MstpType::parse(mstp_bits)
    }

    /// Get message type info (MTIN) - log level, trace type, etc.
    pub fn message_type_info(&self) -> u8 {
        (self.msin >> 4) & 0x0F
    }

    /// Get log level (if message type is Log)
    pub fn log_level(&self) -> Option<MtinTypeDltLog> {
        if matches!(self.message_type(), MstpType::DltTypeLog) {
            Some(MtinTypeDltLog::parse(self.message_type_info()))
        } else {
            None
        }
    }
}

