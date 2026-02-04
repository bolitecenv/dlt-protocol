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

#[derive(Debug, Clone, Copy)]
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
