//! # DLT Verbose Mode Payload Builder and Parser
//!
//! This module provides functionality for building and parsing DLT verbose mode payloads.
//!
//! ## Payload Parser (Re-exported)
//!
//! The payload parser types (`PayloadParser`, `PayloadType`, `TypeLength`, `DltValue`, `PayloadError`)
//! are re-exported from `parse_log` for backward compatibility.
//!
//! ## Payload Builder
//!
//! `PayloadBuilder` remains in this module for creating verbose mode payloads.

// Re-export parser types from parse_log
pub use crate::r19_11::parse_log::{DltValue, PayloadError, PayloadParser, PayloadType, TypeLength};

// ========================================
// Payload Builder (remains here)
// ========================================

/// DLT Payload Builder for no_std environments
/// Uses a fixed-size buffer to avoid heap allocations
pub struct PayloadBuilder<'a> {
    buffer: &'a mut [u8],
    position: usize,
}

impl<'a> PayloadBuilder<'a> {
    /// Create a new payload builder with the given buffer
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    /// Get the number of bytes written
    pub fn len(&self) -> usize {
        self.position
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.position == 0
    }

    /// Get the filled portion of the buffer
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer[..self.position]
    }

    /// Reset the builder to reuse the buffer
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Write raw bytes to the buffer
    fn write_bytes(&mut self, data: &[u8]) -> Result<(), PayloadError> {
        if self.position + data.len() > self.buffer.len() {
            return Err(PayloadError::BufferTooSmall);
        }
        self.buffer[self.position..self.position + data.len()].copy_from_slice(data);
        self.position += data.len();
        Ok(())
    }

    /// Build Type Info field (4 bytes)
    /// Bits 0-3: Type Length (TYLE)
    /// Bit 4: Type Bool (Bool)
    /// Bit 5: Type Signed (SINT)
    /// Bit 6: Type Unsigned (UINT)
    /// Bit 7: Type Float (FLOA)
    /// Bit 8: Type Array (ARAY)
    /// Bit 9: Type String (STRG)
    /// Bit 10: Type Raw (RAWD)
    /// Bit 11: Variable Info (VARI)
    /// Bit 12: Fixed Point (FIXP)
    /// Bit 13: Trace Info (TRAI)
    /// Bit 14: Type Struct (STRU)
    /// Bit 15-17: String Coding (SCOD)
    /// Bit 18-31: reserved
    fn write_type_info(
        &mut self,
        payload_type: PayloadType,
        type_length: TypeLength,
    ) -> Result<(), PayloadError> {
        let type_info: u32 = (type_length as u32) | payload_type.to_bit();
        self.write_bytes(&type_info.to_le_bytes())?;
        Ok(())
    }

    /// Add a boolean value (8 bit)
    pub fn add_bool(&mut self, value: bool) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Bool, TypeLength::Bit8)?;
        self.write_bytes(&[value as u8])?;
        Ok(())
    }

    /// Add a signed 8-bit integer
    pub fn add_i8(&mut self, value: i8) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Signed, TypeLength::Bit8)?;
        self.write_bytes(&value.to_le_bytes())?;
        Ok(())
    }

    /// Add a signed 16-bit integer
    pub fn add_i16(&mut self, value: i16) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Signed, TypeLength::Bit16)?;
        self.write_bytes(&value.to_le_bytes())?;
        Ok(())
    }

    /// Add a signed 32-bit integer
    pub fn add_i32(&mut self, value: i32) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Signed, TypeLength::Bit32)?;
        self.write_bytes(&value.to_le_bytes())?;
        Ok(())
    }

    /// Add a signed 64-bit integer
    pub fn add_i64(&mut self, value: i64) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Signed, TypeLength::Bit64)?;
        self.write_bytes(&value.to_le_bytes())?;
        Ok(())
    }

    /// Add an unsigned 8-bit integer
    pub fn add_u8(&mut self, value: u8) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Unsigned, TypeLength::Bit8)?;
        self.write_bytes(&value.to_le_bytes())?;
        Ok(())
    }

    /// Add an unsigned 16-bit integer
    pub fn add_u16(&mut self, value: u16) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Unsigned, TypeLength::Bit16)?;
        self.write_bytes(&value.to_le_bytes())?;
        Ok(())
    }

    /// Add an unsigned 32-bit integer
    pub fn add_u32(&mut self, value: u32) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Unsigned, TypeLength::Bit32)?;
        self.write_bytes(&value.to_le_bytes())?;
        Ok(())
    }

    /// Add an unsigned 64-bit integer
    pub fn add_u64(&mut self, value: u64) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Unsigned, TypeLength::Bit64)?;
        self.write_bytes(&value.to_le_bytes())?;
        Ok(())
    }

    /// Add a 32-bit float
    pub fn add_f32(&mut self, value: f32) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Float, TypeLength::Bit32)?;
        self.write_bytes(&value.to_le_bytes())?;
        Ok(())
    }

    /// Add a 64-bit float
    pub fn add_f64(&mut self, value: f64) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Float, TypeLength::Bit64)?;
        self.write_bytes(&value.to_le_bytes())?;
        Ok(())
    }

    /// Add a string
    /// For strings, the Type Length field is usually set to NotDefined (0x00)
    /// and the actual length is encoded in the payload
    pub fn add_string(&mut self, value: &str) -> Result<(), PayloadError> {
        // Type info with TYLE = 0 (not defined) for variable length strings
        self.write_type_info(PayloadType::String, TypeLength::NotDefined)?;

        // Write string length (2 bytes)
        let len = (value.len() as u16) + 1; // +1 for null terminator
        self.write_bytes(&len.to_le_bytes())?;

        // Write string data (null-terminated)
        self.write_bytes(value.as_bytes())?;
        self.write_bytes(&[0])?; // null terminator
        Ok(())
    }

    /// Add raw bytes
    pub fn add_raw(&mut self, data: &[u8]) -> Result<(), PayloadError> {
        // Type info with TYLE = 0 (not defined) for variable length raw data
        self.write_type_info(PayloadType::Raw, TypeLength::NotDefined)?;

        // Write data length (2 bytes)
        let len = (data.len() as u16) + 1; // +1 for null terminator
        self.write_bytes(&len.to_le_bytes())?;

        // Write raw data
        self.write_bytes(data)?;
        self.write_bytes(&[0])?; // null terminator
        Ok(())
    }

    /// Add a 128-bit value (generic)
    pub fn add_u128(&mut self, value: u128) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Unsigned, TypeLength::Bit128)?;
        self.write_bytes(&value.to_le_bytes())?;
        Ok(())
    }
}

