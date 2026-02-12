#![cfg_attr(not(target_arch = "wasm32"), no_std)]
#![no_main]

#[cfg(target_arch = "wasm32")]
extern crate std;

#[cfg(target_arch = "wasm32")]
use std::vec::Vec;
#[cfg(target_arch = "wasm32")]
use std::string::String;

use dlt_protocol::r19_11::*;

/// Create a simple DLT log message and return its size
/// This function can be called from JavaScript
#[unsafe(no_mangle)]
pub extern "C" fn create_dlt_message(buffer_ptr: *mut u8, buffer_len: usize) -> usize {
    if buffer_ptr.is_null() || buffer_len < 100 {
        return 0;
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
        Ok(size) => size,
        Err(_) => 0,
    }
}

/// Comprehensive analysis result structure (written to WASM memory)
/// Layout: total 32 bytes
#[repr(C, packed)]
struct AnalysisResult {
    total_len: u16,      // 0-1
    header_len: u16,     // 2-3
    payload_len: u16,    // 4-5
    payload_offset: u16, // 6-7: offset in original buffer where payload starts
    msg_type: u8,        // 8: MTIN (message type info)
    log_level: u8,       // 9: log level (bits 4-7 of MTIN)
    has_serial: u8,      // 10
    has_ecu: u8,         // 11
    ecu_id: [u8; 4],     // 12-15
    app_id: [u8; 4],     // 16-19
    ctx_id: [u8; 4],     // 20-23
    reserved: [u8; 8],   // 24-31: reserved for future use
}

/// Parse a DLT message and write comprehensive analysis results to memory
/// Returns pointer to result struct (32 bytes) or 0 on error
#[unsafe(no_mangle)]
pub extern "C" fn analyze_dlt_message(buffer_ptr: *const u8, buffer_len: usize) -> *mut u8 {
    if buffer_ptr.is_null() || buffer_len < 4 {
        return core::ptr::null_mut();
    }

    let buffer = unsafe { core::slice::from_raw_parts(buffer_ptr, buffer_len) };
    
    let mut offset = 0;
    let has_serial = if buffer.len() >= 4 && &buffer[0..4] == DLT_SERIAL_HEADER_ARRAY {
        offset = 4;
        1u8
    } else {
        0u8
    };

    if buffer.len() < offset + 4 {
        return core::ptr::null_mut();
    }

    let htyp = buffer[offset];
    let has_ecu = if (htyp & WEID_MASK) != 0 { 1u8 } else { 0u8 };
    let has_session = (htyp & WSID_MASK) != 0;
    let has_timestamp = (htyp & WTMS_MASK) != 0;
    let has_extended = (htyp & UEH_MASK) != 0;
    
    // Calculate message length from header
    let len_bytes = [buffer[offset + 2], buffer[offset + 3]];
    let total_len = u16::from_be_bytes(len_bytes);
    
    // Calculate header size
    let mut header_len = 4u16; // Standard header
    if has_ecu != 0 { header_len += 4; }
    if has_session { header_len += 4; }
    if has_timestamp { header_len += 4; }
    if has_extended { header_len += 10; }
    
    let payload_len = if total_len > header_len {
        total_len - header_len
    } else {
        0
    };

    // Extract ECU ID
    let mut ecu_id = [0u8; 4];
    if has_ecu != 0 && buffer.len() >= offset + 8 {
        ecu_id.copy_from_slice(&buffer[offset + 4..offset + 8]);
    }

    // Calculate offset to extended header
    let mut ext_offset = offset + 4;
    if has_ecu != 0 { ext_offset += 4; }
    if has_session { ext_offset += 4; }
    if has_timestamp { ext_offset += 4; }

    // Extract message type, log level, App ID, Context ID
    let mut msg_type = 0u8;
    let mut log_level = 0u8;
    let mut app_id = [0u8; 4];
    let mut ctx_id = [0u8; 4];
    
    if has_extended && buffer.len() >= ext_offset + 10 {
        msg_type = buffer[ext_offset]; // MSIN byte
        log_level = (msg_type >> 4) & 0x0F; // Extract log level from upper 4 bits
        
        if buffer.len() >= ext_offset + 8 {
            app_id.copy_from_slice(&buffer[ext_offset + 4..ext_offset + 8]);
        }
        if buffer.len() >= ext_offset + 12 {
            ctx_id.copy_from_slice(&buffer[ext_offset + 8..ext_offset + 12]);
        }
    }

    let payload_offset = (offset + header_len as usize) as u16;

    // Allocate and write result
    let result_ptr = allocate(32); // Size of AnalysisResult
    if result_ptr.is_null() {
        return core::ptr::null_mut();
    }

    unsafe {
        let result = result_ptr as *mut AnalysisResult;
        (*result).total_len = total_len;
        (*result).header_len = header_len;
        (*result).payload_len = payload_len;
        (*result).payload_offset = payload_offset;
        (*result).msg_type = msg_type;
        (*result).log_level = log_level;
        (*result).has_serial = has_serial;
        (*result).has_ecu = has_ecu;
        (*result).ecu_id = ecu_id;
        (*result).app_id = app_id;
        (*result).ctx_id = ctx_id;
        (*result).reserved = [0; 8];
    }

    result_ptr
}

#[cfg(target_arch = "wasm32")]
static mut FORMATTED_PAYLOAD: Option<Vec<u8>> = None;

/// Parse verbose payload and format if first argument is a string with {}
/// Returns length of formatted string, or 0 on error
/// Result is stored in a global Vec that can be accessed via get_formatted_payload_ptr
#[unsafe(no_mangle)]
pub extern "C" fn format_verbose_payload(buffer_ptr: *const u8, buffer_len: usize, payload_offset: u16, payload_len: u16) -> usize {
    #[cfg(not(target_arch = "wasm32"))]
    {
        return 0;
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        if buffer_ptr.is_null() || payload_len == 0 {
            return 0;
        }

        let buffer = unsafe { core::slice::from_raw_parts(buffer_ptr, buffer_len) };
        let offset = payload_offset as usize;
        
        if offset + payload_len as usize > buffer_len {
            return 0;
        }

        let payload = &buffer[offset..offset + payload_len as usize];
        
        // Try to parse verbose payload
        if payload.len() < 4 {
            return 0;
        }

        let type_info = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
        let is_string = (type_info & 0x0200) != 0; // STRG_ASCII flag
        
        if !is_string {
            // Not a string, just return raw payload as text
            let text: Vec<u8> = payload.iter()
                .filter(|&&b| b >= 32 && b < 127)
                .copied()
                .collect();
            let len = text.len();
            unsafe {
                FORMATTED_PAYLOAD = Some(text);
            }
            return len;
        }

        // Parse string argument
        let mut pos = 4; // Skip type info
        if pos + 2 > payload.len() {
            return 0;
        }

        let str_len = u16::from_le_bytes([payload[pos], payload[pos + 1]]) as usize;
        pos += 2;

        if pos + str_len > payload.len() {
            return 0;
        }

        let format_str = &payload[pos..pos + str_len];
        pos += str_len;

        // Check if format string contains "{}"
        let format_string = String::from_utf8_lossy(format_str);
        
        if !format_string.contains("{}") {
            // No placeholders, just return the string
            unsafe {
                FORMATTED_PAYLOAD = Some(format_str.to_vec());
            }
            return format_str.len();
        }

        // Parse next argument (assume it's an integer)
        if pos + 4 > payload.len() {
            // No more arguments, return format string as-is
            unsafe {
                FORMATTED_PAYLOAD = Some(format_str.to_vec());
            }
            return format_str.len();
        }

        let arg_type_info = u32::from_le_bytes([payload[pos], payload[pos + 1], payload[pos + 2], payload[pos + 3]]);
        pos += 4;

        // Extract type length (bits 0-3)
        let type_length = (arg_type_info & 0x0F) as usize;
        let arg_size = match type_length {
            1 => 1, // 8 bit
            2 => 2, // 16 bit
            3 => 4, // 32 bit
            4 => 8, // 64 bit
            5 => 16, // 128 bit
            _ => 4, // default to 32 bit
        };

        if pos + arg_size > payload.len() {
            unsafe {
                FORMATTED_PAYLOAD = Some(format_str.to_vec());
            }
            return format_str.len();
        }

        // Parse integer value (support signed/unsigned)
        let is_signed = (arg_type_info & 0x1000) != 0; // SINT flag
        let value: i64 = match arg_size {
            1 => {
                if is_signed {
                    payload[pos] as i8 as i64
                } else {
                    payload[pos] as i64
                }
            }
            2 => {
                let val = u16::from_le_bytes([payload[pos], payload[pos + 1]]);
                if is_signed {
                    val as i16 as i64
                } else {
                    val as i64
                }
            }
            4 => {
                let val = u32::from_le_bytes([payload[pos], payload[pos + 1], payload[pos + 2], payload[pos + 3]]);
                if is_signed {
                    val as i32 as i64
                } else {
                    val as i64
                }
            }
            8 => {
                let val = u64::from_le_bytes([
                    payload[pos], payload[pos + 1], payload[pos + 2], payload[pos + 3],
                    payload[pos + 4], payload[pos + 5], payload[pos + 6], payload[pos + 7],
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
        
        len
    }
}

/// Get pointer to formatted payload buffer
#[unsafe(no_mangle)]
pub extern "C" fn get_formatted_payload_ptr() -> *const u8 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        core::ptr::addr_of!(FORMATTED_PAYLOAD)
            .read()
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

    if buffer.len() < offset + 4 {
        return 0;
    }

    let htyp = buffer[offset];
    if (htyp & WEID_MASK) == 0 {
        return 0; // No ECU ID
    }

    if buffer.len() < offset + 8 {
        return 0;
    }

    u32::from_le_bytes([
        buffer[offset + 4],
        buffer[offset + 5],
        buffer[offset + 6],
        buffer[offset + 7],
    ])
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

    if buffer.len() < offset + 4 {
        return 0;
    }

    let htyp = buffer[offset];
    if (htyp & UEH_MASK) == 0 {
        return 0; // No extended header
    }

    let mut ext_offset = offset + 4; // Standard header
    if (htyp & WEID_MASK) != 0 { ext_offset += 4; }
    if (htyp & WSID_MASK) != 0 { ext_offset += 4; }
    if (htyp & WTMS_MASK) != 0 { ext_offset += 4; }

    if buffer.len() < ext_offset + 8 {
        return 0;
    }

    u32::from_le_bytes([
        buffer[ext_offset + 4],
        buffer[ext_offset + 5],
        buffer[ext_offset + 6],
        buffer[ext_offset + 7],
    ])
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

    if buffer.len() < offset + 4 {
        return 0;
    }

    let htyp = buffer[offset];
    if (htyp & UEH_MASK) == 0 {
        return 0; // No extended header
    }

    // Calculate offset to extended header
    let mut ext_offset = offset + 4;
    if (htyp & WEID_MASK) != 0 { ext_offset += 4; }
    if (htyp & WSID_MASK) != 0 { ext_offset += 4; }
    if (htyp & WTMS_MASK) != 0 { ext_offset += 4; }

    if buffer.len() < ext_offset + 12 {
        return 0;
    }

    u32::from_le_bytes([
        buffer[ext_offset + 8],
        buffer[ext_offset + 9],
        buffer[ext_offset + 10],
        buffer[ext_offset + 11],
    ])
}

/// Allocate memory for WASM (simple bump allocator for demo)
static mut HEAP: [u8; 4096] = [0; 4096];
static mut HEAP_POS: usize = 0;

#[unsafe(no_mangle)]
pub extern "C" fn allocate(size: usize) -> *mut u8 {
    unsafe {
        let heap_len = core::ptr::addr_of!(HEAP).read().len();
        let current_pos = core::ptr::addr_of!(HEAP_POS).read();
        
        if current_pos + size > heap_len {
            return core::ptr::null_mut();
        }
        
        let ptr = core::ptr::addr_of_mut!(HEAP).cast::<u8>().add(current_pos);
        core::ptr::addr_of_mut!(HEAP_POS).write(current_pos + size);
        ptr
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn reset_allocator() {
    unsafe {
        core::ptr::addr_of_mut!(HEAP_POS).write(0);
    }
}
