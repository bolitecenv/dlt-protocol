#![cfg_attr(not(target_arch = "wasm32"), no_std)]
#![no_main]

#[cfg(target_arch = "wasm32")]
extern crate std;

#[cfg(target_arch = "wasm32")]
use std::vec::Vec;
#[cfg(target_arch = "wasm32")]
use std::string::String;

use dlt_protocol::r19_11::*;

// Error codes
pub const ERROR_NULL_POINTER: i32 = -1;
pub const ERROR_BUFFER_TOO_SMALL: i32 = -2;
pub const ERROR_INVALID_FORMAT: i32 = -3;
pub const ERROR_OUT_OF_MEMORY: i32 = -4;

/// Create a simple DLT log message and return its size
/// Returns positive size on success, negative error code on failure
#[unsafe(no_mangle)]
pub extern "C" fn create_dlt_message(buffer_ptr: *mut u8, buffer_len: usize) -> i32 {
    if buffer_ptr.is_null() {
        return ERROR_NULL_POINTER;
    }
    if buffer_len < 100 {
        return ERROR_BUFFER_TOO_SMALL;
    }

    let buffer = unsafe { core::slice::from_raw_parts_mut(buffer_ptr, buffer_len) };
    
    let mut builder = DltMessageBuilder::new()
        .with_ecu_id(b"WASM")
        .with_app_id(b"TEST")
        .with_context_id(b"DEMO")
        .add_serial_header();

    let payload = b"Hello from WASM!";
    
    match builder.generate_log_message_with_payload(
        buffer,
        payload,
        MtinTypeDltLog::DltLogInfo,
        1,
        false, // non-verbose mode
    ) {
        Ok(size) => size as i32,
        Err(_) => ERROR_INVALID_FORMAT,
    }
}

/// Comprehensive analysis result structure (written to WASM memory)
/// Layout: total 32 bytes
#[repr(C)]
#[derive(Clone, Copy)]
struct AnalysisResult {
    total_len: u16,      // 0-1
    header_len: u16,     // 2-3
    payload_len: u16,    // 4-5
    payload_offset: u16, // 6-7: offset in original buffer where payload starts
    msg_type: u8,        // 8: MSIN (message type info)
    log_level: u8,       // 9: log level (bits 1-3 of MTIN for log messages)
    has_serial: u8,      // 10
    has_ecu: u8,         // 11
    ecu_id: [u8; 4],     // 12-15
    app_id: [u8; 4],     // 16-19
    ctx_id: [u8; 4],     // 20-23
    mstp: u8,            // 24: Message Type (0=Log, 1=Trace, 2=Network, 3=Control)
    is_verbose: u8,      // 25: Verbose mode flag (bit 0 of MSIN)
    reserved: [u8; 6],   // 26-31: reserved for future use
}

/// Safe slice helper function
fn safe_slice(buffer: &[u8], start: usize, len: usize) -> Option<&[u8]> {
    let end = start.checked_add(len)?;
    if end <= buffer.len() {
        Some(&buffer[start..end])
    } else {
        None
    }
}

/// Parse a DLT message using r19-11 DltHeaderParser and write comprehensive analysis results to memory
/// Returns pointer to result struct (32 bytes) or null on error
#[unsafe(no_mangle)]
pub extern "C" fn analyze_dlt_message(buffer_ptr: *const u8, buffer_len: usize) -> *mut u8 {
    if buffer_ptr.is_null() || buffer_len < 4 {
        return core::ptr::null_mut();
    }

    let buffer = unsafe { core::slice::from_raw_parts(buffer_ptr, buffer_len) };
    
    // Use r19-11 DltHeaderParser to parse the message
    let mut parser = DltHeaderParser::new(buffer);
    let parsed_msg = match parser.parse_message() {
        Ok(msg) => msg,
        Err(_) => return core::ptr::null_mut(),
    };

    // Extract values from parsed message
    let has_serial = if parsed_msg.has_serial_header { 1u8 } else { 0u8 };
    let has_ecu = if parsed_msg.ecu_id.is_some() { 1u8 } else { 0u8 };
    let ecu_id = parsed_msg.ecu_id.unwrap_or([0u8; 4]);
    
    let total_len = parsed_msg.standard_header.len;
    
    // Calculate header length
    let mut header_len = 4u16; // Standard header
    if parsed_msg.ecu_id.is_some() { header_len += 4; }
    if parsed_msg.session_id.is_some() { header_len += 4; }
    if parsed_msg.timestamp.is_some() { header_len += 4; }
    if parsed_msg.extended_header.is_some() { header_len += 10; }
    
    let payload_len = parsed_msg.payload.len() as u16;
    
    // Calculate payload offset in original buffer
    let serial_offset = if parsed_msg.has_serial_header { 4 } else { 0 };
    let payload_offset = (serial_offset + header_len) as u16;

    // Extract message type information from extended header using r19-11 parsers
    let (msg_type, mstp, log_level, is_verbose, app_id, ctx_id) = if let Some(ext_hdr) = parsed_msg.extended_header {
        let msin = ext_hdr.msin;
        
        // Use r19-11's MstpType::parse to extract message type from bits 7-4
        let mstp_raw = (msin >> 4) & 0x0F;
        let mstp_type = MstpType::parse(mstp_raw);
        
        // Use r19-11's Mtin::parse to properly decode MTIN bits based on MSTP
        let mtin_raw = (msin >> 1) & 0x07;
        let mtin = Mtin::parse(&mstp_type, mtin_raw);
        
        // Extract verbose flag (bit 0)
        let verbose = msin & 0x01;
        
        // Get log level using r19-11's type matching
        let log_level = match mtin {
            Mtin::Log(log_type) => {
                // Convert MtinTypeDltLog to numeric level
                match log_type {
                    MtinTypeDltLog::DltLogFatal => 1,
                    MtinTypeDltLog::DltLogError => 2,
                    MtinTypeDltLog::DltLogWarn => 3,
                    MtinTypeDltLog::DltLogInfo => 4,
                    MtinTypeDltLog::DltLogDebug => 5,
                    MtinTypeDltLog::DltLogVerbose => 6,
                    _ => 0,
                }
            },
            _ => 0, // Non-log messages have no log level
        };
        
        (msin, mstp_type.to_bits(), log_level, verbose, ext_hdr.apid, ext_hdr.ctid)
    } else {
        (0u8, 0u8, 0u8, 0u8, [0u8; 4], [0u8; 4])
    };

    // Allocate result buffer (32 bytes)
    let result_ptr = allocate(32);
    if result_ptr.is_null() {
        return core::ptr::null_mut();
    }

    // Write result fields manually byte-by-byte to guarantee layout matches JavaScript
    // Layout: [total_len:u16LE][header_len:u16LE][payload_len:u16LE][payload_offset:u16LE]
    //         [msg_type:u8][log_level:u8][has_serial:u8][has_ecu:u8]
    //         [ecu_id:4][app_id:4][ctx_id:4]
    //         [mstp:u8][is_verbose:u8][reserved:6]
    unsafe {
        let p = result_ptr;
        // Offset 0-1: total_len (u16 little-endian)
        *p.add(0) = (total_len & 0xFF) as u8;
        *p.add(1) = ((total_len >> 8) & 0xFF) as u8;
        // Offset 2-3: header_len (u16 little-endian)
        *p.add(2) = (header_len & 0xFF) as u8;
        *p.add(3) = ((header_len >> 8) & 0xFF) as u8;
        // Offset 4-5: payload_len (u16 little-endian)
        *p.add(4) = (payload_len & 0xFF) as u8;
        *p.add(5) = ((payload_len >> 8) & 0xFF) as u8;
        // Offset 6-7: payload_offset (u16 little-endian)
        *p.add(6) = (payload_offset & 0xFF) as u8;
        *p.add(7) = ((payload_offset >> 8) & 0xFF) as u8;
        // Offset 8: msg_type
        *p.add(8) = msg_type;
        // Offset 9: log_level
        *p.add(9) = log_level;
        // Offset 10: has_serial
        *p.add(10) = has_serial;
        // Offset 11: has_ecu
        *p.add(11) = has_ecu;
        // Offset 12-15: ecu_id
        *p.add(12) = ecu_id[0];
        *p.add(13) = ecu_id[1];
        *p.add(14) = ecu_id[2];
        *p.add(15) = ecu_id[3];
        // Offset 16-19: app_id
        *p.add(16) = app_id[0];
        *p.add(17) = app_id[1];
        *p.add(18) = app_id[2];
        *p.add(19) = app_id[3];
        // Offset 20-23: ctx_id
        *p.add(20) = ctx_id[0];
        *p.add(21) = ctx_id[1];
        *p.add(22) = ctx_id[2];
        *p.add(23) = ctx_id[3];
        // Offset 24: mstp
        *p.add(24) = mstp;
        // Offset 25: is_verbose
        *p.add(25) = is_verbose;
        // Offset 26-31: reserved (zero)
        *p.add(26) = 0;
        *p.add(27) = 0;
        *p.add(28) = 0;
        *p.add(29) = 0;
        *p.add(30) = 0;
        *p.add(31) = 0;
    }

    result_ptr
}

#[cfg(target_arch = "wasm32")]
static mut FORMATTED_PAYLOAD: Option<Vec<u8>> = None;

/// Parse verbose payload ONLY for Log messages (MSTP=0) and format if first argument is a string with {}
/// For Service/Control or Network messages, returns error code
/// Returns length of formatted string, or negative error code on error
/// Result is stored in a global Vec that can be accessed via get_formatted_payload_ptr
#[unsafe(no_mangle)]
pub extern "C" fn format_verbose_payload(
    buffer_ptr: *const u8, 
    buffer_len: usize, 
    payload_offset: u16, 
    payload_len: u16,
    mstp: u8  // Add message type parameter
) -> i32 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        return ERROR_INVALID_FORMAT;
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        if buffer_ptr.is_null() {
            return ERROR_NULL_POINTER;
        }
        if payload_len == 0 {
            return ERROR_BUFFER_TOO_SMALL;
        }

        // Only parse verbose payload for Log messages (MSTP=0)
        if mstp != 0 {
            // For Service/Control (3) or Network (2) messages, return a special code
            // indicating no payload parsing
            return ERROR_INVALID_FORMAT;
        }

        let buffer = unsafe { core::slice::from_raw_parts(buffer_ptr, buffer_len) };
        let offset = payload_offset as usize;
        
        let payload = match safe_slice(buffer, offset, payload_len as usize) {
            Some(p) => p,
            None => return ERROR_BUFFER_TOO_SMALL,
        };
        
        // Try to parse verbose payload
        if payload.len() < 4 {
            return ERROR_INVALID_FORMAT;
        }

        let type_info_slice = match safe_slice(payload, 0, 4) {
            Some(s) => s,
            None => return ERROR_INVALID_FORMAT,
        };
        let type_info = u32::from_le_bytes([
            type_info_slice[0], 
            type_info_slice[1], 
            type_info_slice[2], 
            type_info_slice[3]
        ]);
        let is_string = (type_info & 0x0200) != 0; // STRG_ASCII flag
        
        if !is_string {
            // Not a string, just return printable characters from raw payload
            let text: Vec<u8> = payload.iter()
                .filter(|&&b| b >= 32 && b < 127)
                .copied()
                .collect();
            let len = text.len();
            unsafe {
                FORMATTED_PAYLOAD = Some(text);
            }
            return len as i32;
        }

        // Parse string argument
        let mut pos = 4; // Skip type info
        
        let str_len_slice = match safe_slice(payload, pos, 2) {
            Some(s) => s,
            None => return ERROR_INVALID_FORMAT,
        };
        let str_len = u16::from_le_bytes([str_len_slice[0], str_len_slice[1]]) as usize;
        pos += 2;

        let format_str = match safe_slice(payload, pos, str_len) {
            Some(s) => s,
            None => return ERROR_INVALID_FORMAT,
        };
        pos += str_len;

        // Check if format string contains "{}"
        let format_string = String::from_utf8_lossy(format_str);
        
        if !format_string.contains("{}") {
            // No placeholders, just return the string
            unsafe {
                FORMATTED_PAYLOAD = Some(format_str.to_vec());
            }
            return format_str.len() as i32;
        }

        // Parse next argument (assume it's an integer)
        if pos + 4 > payload.len() {
            // No more arguments, return format string as-is
            unsafe {
                FORMATTED_PAYLOAD = Some(format_str.to_vec());
            }
            return format_str.len() as i32;
        }

        let arg_type_slice = match safe_slice(payload, pos, 4) {
            Some(s) => s,
            None => return ERROR_INVALID_FORMAT,
        };
        let arg_type_info = u32::from_le_bytes([
            arg_type_slice[0], 
            arg_type_slice[1], 
            arg_type_slice[2], 
            arg_type_slice[3]
        ]);
        pos += 4;

        // Extract type length (bits 0-3)
        let type_length = (arg_type_info & 0x0F) as usize;
        let arg_size = match type_length {
            1 => 1,  // 8 bit
            2 => 2,  // 16 bit
            3 => 4,  // 32 bit
            4 => 8,  // 64 bit
            5 => 16, // 128 bit
            _ => 4,  // default to 32 bit
        };

        let arg_data = match safe_slice(payload, pos, arg_size) {
            Some(data) => data,
            None => {
                unsafe {
                    FORMATTED_PAYLOAD = Some(format_str.to_vec());
                }
                return format_str.len() as i32;
            }
        };

        // Parse integer value (support signed/unsigned)
        let is_signed = (arg_type_info & 0x1000) != 0; // SINT flag
        let value: i64 = match arg_size {
            1 => {
                if is_signed {
                    arg_data[0] as i8 as i64
                } else {
                    arg_data[0] as i64
                }
            }
            2 => {
                let val = u16::from_le_bytes([arg_data[0], arg_data[1]]);
                if is_signed {
                    val as i16 as i64
                } else {
                    val as i64
                }
            }
            4 => {
                let val = u32::from_le_bytes([
                    arg_data[0], arg_data[1], arg_data[2], arg_data[3]
                ]);
                if is_signed {
                    val as i32 as i64
                } else {
                    val as i64
                }
            }
            8 => {
                let val = u64::from_le_bytes([
                    arg_data[0], arg_data[1], arg_data[2], arg_data[3],
                    arg_data[4], arg_data[5], arg_data[6], arg_data[7],
                ]);
                val as i64
            }
            _ => 0,
        };

        // Format the string using String::replace
        let formatted = format_string.replacen("{}", &value.to_string(), 1);
        let formatted_bytes = formatted.as_bytes().to_vec();
        let len = formatted_bytes.len();
        
        unsafe {
            FORMATTED_PAYLOAD = Some(formatted_bytes);
        }
        
        len as i32
    }
}

/// Get pointer to formatted payload buffer
#[unsafe(no_mangle)]
pub extern "C" fn get_formatted_payload_ptr() -> *const u8 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        let payload_ptr = core::ptr::addr_of!(FORMATTED_PAYLOAD);
        (*payload_ptr)
            .as_ref()
            .map(|v| v.as_ptr())
            .unwrap_or(core::ptr::null())
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    core::ptr::null()
}

/// Get the version info
#[unsafe(no_mangle)]
pub extern "C" fn get_version() -> u32 {
    100 // Version 1.0.0
}

/// Extract ECU ID from DLT message
/// Returns 0 if no ECU ID, otherwise returns 4-byte ECU ID as u32
#[unsafe(no_mangle)]
pub extern "C" fn get_ecu_id(buffer_ptr: *const u8, buffer_len: usize) -> u32 {
    if buffer_ptr.is_null() || buffer_len < 4 {
        return 0;
    }

    let buffer = unsafe { core::slice::from_raw_parts(buffer_ptr, buffer_len) };
    
    let mut offset = 0;
    if buffer.len() >= 4 && &buffer[0..4] == DLT_SERIAL_HEADER_ARRAY {
        offset = 4;
    }

    let htyp_slice = match safe_slice(buffer, offset, 1) {
        Some(s) => s,
        None => return 0,
    };
    let htyp = *htyp_slice.get(0).unwrap_or(&0);
    
    if (htyp & WEID_MASK) == 0 {
        return 0; // No ECU ID
    }

    let ecu_slice = match safe_slice(buffer, offset + 4, 4) {
        Some(s) => s,
        None => return 0,
    };
    Some(u32::from_le_bytes([
        ecu_slice[0],
        ecu_slice[1],
        ecu_slice[2],
        ecu_slice[3],
    ])).unwrap_or(0)
}

/// Extract App ID from DLT message (from extended header)
#[unsafe(no_mangle)]
pub extern "C" fn get_app_id(buffer_ptr: *const u8, buffer_len: usize) -> u32 {
    if buffer_ptr.is_null() || buffer_len < 4 {
        return 0;
    }

    let buffer = unsafe { core::slice::from_raw_parts(buffer_ptr, buffer_len) };
    
    let mut offset = 0;
    if buffer.len() >= 4 && &buffer[0..4] == DLT_SERIAL_HEADER_ARRAY {
        offset = 4;
    }

    let htyp_slice = match safe_slice(buffer, offset, 1) {
        Some(s) => s,
        None => return 0,
    };
    let htyp = *htyp_slice.get(0).unwrap_or(&0);
    
    if (htyp & UEH_MASK) == 0 {
        return 0; // No extended header
    }

    let mut ext_offset = offset + 4; // Standard header
    if (htyp & WEID_MASK) != 0 { ext_offset += 4; }
    if (htyp & WSID_MASK) != 0 { ext_offset += 4; }
    if (htyp & WTMS_MASK) != 0 { ext_offset += 4; }

    let app_slice = match safe_slice(buffer, ext_offset + 4, 4) {
        Some(s) => s,
        None => return 0,
    };
    Some(u32::from_le_bytes([
        app_slice[0],
        app_slice[1],
        app_slice[2],
        app_slice[3],
    ])).unwrap_or(0)
}

/// Extract Context ID from DLT message (from extended header)
#[unsafe(no_mangle)]
pub extern "C" fn get_context_id(buffer_ptr: *const u8, buffer_len: usize) -> u32 {
    if buffer_ptr.is_null() || buffer_len < 4 {
        return 0;
    }

    let buffer = unsafe { core::slice::from_raw_parts(buffer_ptr, buffer_len) };
    
    let mut offset = 0;
    if buffer.len() >= 4 && &buffer[0..4] == DLT_SERIAL_HEADER_ARRAY {
        offset = 4;
    }

    let htyp_slice = match safe_slice(buffer, offset, 1) {
        Some(s) => s,
        None => return 0,
    };
    let htyp = *htyp_slice.get(0).unwrap_or(&0);
    
    if (htyp & UEH_MASK) == 0 {
        return 0; // No extended header
    }

    // Calculate offset to extended header
    let mut ext_offset = offset + 4;
    if (htyp & WEID_MASK) != 0 { ext_offset += 4; }
    if (htyp & WSID_MASK) != 0 { ext_offset += 4; }
    if (htyp & WTMS_MASK) != 0 { ext_offset += 4; }

    let ctx_slice = match safe_slice(buffer, ext_offset + 8, 4) {
        Some(s) => s,
        None => return 0,
    };
    Some(u32::from_le_bytes([
        ctx_slice[0],
        ctx_slice[1],
        ctx_slice[2],
        ctx_slice[3],
    ])).unwrap_or(0)
}

/// Improved allocator with metadata tracking
#[repr(C)]
struct AllocHeader {
    size: usize,
    in_use: u8,
}

const HEADER_SIZE: usize = core::mem::size_of::<AllocHeader>();
static mut HEAP: [u8; 8192] = [0; 8192]; // Increased from 4096
static mut HEAP_POS: usize = 0;

/// Allocate memory for WASM with size tracking
#[unsafe(no_mangle)]
pub extern "C" fn allocate(size: usize) -> *mut u8 {
    unsafe {
        let heap_ptr = core::ptr::addr_of_mut!(HEAP);
        let heap_pos_ptr = core::ptr::addr_of_mut!(HEAP_POS);
        let heap_len = (*heap_ptr).len();
        let current_pos = *heap_pos_ptr;
        
        let aligned_size = (size + 7) & !7; // 8-byte alignment
        let total_size = HEADER_SIZE + aligned_size;
        
        if current_pos + total_size > heap_len {
            return core::ptr::null_mut();
        }
        
        let header_ptr = (*heap_ptr).as_mut_ptr().add(current_pos) as *mut AllocHeader;
        core::ptr::write_unaligned(header_ptr, AllocHeader {
            size: aligned_size,
            in_use: 1,
        });
        
        let data_ptr = (*heap_ptr).as_mut_ptr().add(current_pos + HEADER_SIZE);
        *heap_pos_ptr = current_pos + total_size;
        
        data_ptr
    }
}

/// Deallocate memory (marks as free but doesn't compact)
#[unsafe(no_mangle)]
pub extern "C" fn deallocate(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    
    unsafe {
        let heap_ptr = core::ptr::addr_of!(HEAP);
        let heap_start = (*heap_ptr).as_ptr() as usize;
        let heap_end = heap_start + (*heap_ptr).len();
        let ptr_addr = ptr as usize;
        
        // Validate pointer is within heap
        if ptr_addr < heap_start + HEADER_SIZE || ptr_addr >= heap_end {
            return;
        }
        
        let header_ptr = ptr.sub(HEADER_SIZE) as *mut AllocHeader;
        let mut header = core::ptr::read_unaligned(header_ptr);
        header.in_use = 0;
        core::ptr::write_unaligned(header_ptr, header);
    }
}

/// Reset allocator (clears all allocations)
#[unsafe(no_mangle)]
pub extern "C" fn reset_allocator() {
    unsafe {
        let heap_pos_ptr = core::ptr::addr_of_mut!(HEAP_POS);
        let heap_ptr = core::ptr::addr_of_mut!(HEAP);
        *heap_pos_ptr = 0;
        // Clear the heap for security
        (*heap_ptr).fill(0);
    }
}

/// Get current heap usage
#[unsafe(no_mangle)]
pub extern "C" fn get_heap_usage() -> usize {
    unsafe {
        let heap_pos_ptr = core::ptr::addr_of!(HEAP_POS);
        *heap_pos_ptr
    }
}

/// Get total heap capacity
#[unsafe(no_mangle)]
pub extern "C" fn get_heap_capacity() -> usize {
    unsafe {
        let heap_ptr = core::ptr::addr_of!(HEAP);
        (*heap_ptr).len()
    }
}