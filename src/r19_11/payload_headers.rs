#![no_std]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadType {
    Bool,
    Signed,
    Unsigned,
    Float,
    Array,
    String,
    Raw,
    VariableInfo,
    FixedPoint,
    TraceInfo,
    Struct,
    StringCoding,
    Reserved,
    Invalid,
}

/// Type Length encoding as per PRS_Dlt_00626
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeLength {
    NotDefined = 0x00,
    Bit8 = 0x01,
    Bit16 = 0x02,
    Bit32 = 0x03,
    Bit64 = 0x04,
    Bit128 = 0x05,
}

impl TypeLength {
    pub fn from_bytes(bytes: usize) -> Option<Self> {
        match bytes {
            1 => Some(TypeLength::Bit8),
            2 => Some(TypeLength::Bit16),
            4 => Some(TypeLength::Bit32),
            8 => Some(TypeLength::Bit64),
            16 => Some(TypeLength::Bit128),
            _ => None,
        }
    }

    pub fn to_bytes(self) -> usize {
        match self {
            TypeLength::NotDefined => 0,
            TypeLength::Bit8 => 1,
            TypeLength::Bit16 => 2,
            TypeLength::Bit32 => 4,
            TypeLength::Bit64 => 8,
            TypeLength::Bit128 => 16,
        }
    }
}

impl PayloadType {
    pub fn parse(type_info: u32) -> Option<Self> {
        if type_info & (1 << 4) != 0 {
            return Some(PayloadType::Bool);
        }
        if type_info & (1 << 5) != 0 {
            return Some(PayloadType::Signed);
        }
        if type_info & (1 << 6) != 0 {
            return Some(PayloadType::Unsigned);
        }
        if type_info & (1 << 7) != 0 {
            return Some(PayloadType::Float);
        }
        if type_info & (1 << 8) != 0 {
            return Some(PayloadType::Array);
        }
        if type_info & (1 << 9) != 0 {
            return Some(PayloadType::String);
        }
        if type_info & (1 << 10) != 0 {
            return Some(PayloadType::Raw);
        }
        if type_info & (1 << 11) != 0 {
            return Some(PayloadType::VariableInfo);
        }
        if type_info & (1 << 12) != 0 {
            return Some(PayloadType::FixedPoint);
        }
        if type_info & (1 << 13) != 0 {
            return Some(PayloadType::TraceInfo);
        }
        if type_info & (1 << 14) != 0 {
            return Some(PayloadType::Struct);
        }
        if type_info & (1 << 15) != 0 {
            return Some(PayloadType::StringCoding);
        }
        None
    }

    pub fn to_bit(self) -> u32 {
        match self {
            PayloadType::Bool => 1 << 4,
            PayloadType::Signed => 1 << 5,
            PayloadType::Unsigned => 1 << 6,
            PayloadType::Float => 1 << 7,
            PayloadType::Array => 1 << 8,
            PayloadType::String => 1 << 9,
            PayloadType::Raw => 1 << 10,
            PayloadType::VariableInfo => 1 << 11,
            PayloadType::FixedPoint => 1 << 12,
            PayloadType::TraceInfo => 1 << 13,
            PayloadType::Struct => 1 << 14,
            PayloadType::StringCoding => 1 << 15,
            _ => 0,
        }
    }
}

#[derive(Debug)]
pub enum PayloadError {
    BufferTooSmall,
    InvalidType,
    InvalidData,
    UnsupportedLength,
}

/// Represents a parsed DLT payload value
/// This enum allows parsing unknown payload types from incoming packets
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DltValue<'a> {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
    String(&'a str),
    Raw(&'a [u8]),
}

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

/// Payload parser for reading DLT payloads
pub struct PayloadParser<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> PayloadParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }

    pub fn is_empty(&self) -> bool {
        self.position >= self.data.len()
    }

    fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], PayloadError> {
        if self.position + count > self.data.len() {
            return Err(PayloadError::BufferTooSmall);
        }
        let slice = &self.data[self.position..self.position + count];
        self.position += count;
        Ok(slice)
    }

    /// Read and parse the next type info field
    pub fn read_type_info(&mut self) -> Result<(PayloadType, TypeLength), PayloadError> {
        let bytes = self.read_bytes(4)?;
        let type_info = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let type_length = match type_info & 0x0F {
            0x00 => TypeLength::NotDefined,
            0x01 => TypeLength::Bit8,
            0x02 => TypeLength::Bit16,
            0x03 => TypeLength::Bit32,
            0x04 => TypeLength::Bit64,
            0x05 => TypeLength::Bit128,
            _ => return Err(PayloadError::InvalidType),
        };

        let payload_type = PayloadType::parse(type_info).ok_or(PayloadError::InvalidType)?;

        Ok((payload_type, type_length))
    }

    /// Read a boolean value
    pub fn read_bool(&mut self) -> Result<bool, PayloadError> {
        let (ptype, tlen) = self.read_type_info()?;
        if ptype != PayloadType::Bool || tlen != TypeLength::Bit8 {
            return Err(PayloadError::InvalidType);
        }
        let bytes = self.read_bytes(1)?;
        Ok(bytes[0] != 0)
    }

    /// Read a signed 8-bit integer
    pub fn read_i8(&mut self) -> Result<i8, PayloadError> {
        let (ptype, tlen) = self.read_type_info()?;
        if ptype != PayloadType::Signed || tlen != TypeLength::Bit8 {
            return Err(PayloadError::InvalidType);
        }
        let bytes = self.read_bytes(1)?;
        Ok(i8::from_le_bytes([bytes[0]]))
    }

    /// Read a signed 16-bit integer
    pub fn read_i16(&mut self) -> Result<i16, PayloadError> {
        let (ptype, tlen) = self.read_type_info()?;
        if ptype != PayloadType::Signed || tlen != TypeLength::Bit16 {
            return Err(PayloadError::InvalidType);
        }
        let bytes = self.read_bytes(2)?;
        Ok(i16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// Read a signed 32-bit integer
    pub fn read_i32(&mut self) -> Result<i32, PayloadError> {
        let (ptype, tlen) = self.read_type_info()?;
        if ptype != PayloadType::Signed || tlen != TypeLength::Bit32 {
            return Err(PayloadError::InvalidType);
        }
        let bytes = self.read_bytes(4)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read a signed 64-bit integer
    pub fn read_i64(&mut self) -> Result<i64, PayloadError> {
        let (ptype, tlen) = self.read_type_info()?;
        if ptype != PayloadType::Signed || tlen != TypeLength::Bit64 {
            return Err(PayloadError::InvalidType);
        }
        let bytes = self.read_bytes(8)?;
        Ok(i64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Read an unsigned 8-bit integer
    pub fn read_u8(&mut self) -> Result<u8, PayloadError> {
        let (ptype, tlen) = self.read_type_info()?;
        if ptype != PayloadType::Unsigned || tlen != TypeLength::Bit8 {
            return Err(PayloadError::InvalidType);
        }
        let bytes = self.read_bytes(1)?;
        Ok(bytes[0])
    }

    /// Read an unsigned 16-bit integer
    pub fn read_u16(&mut self) -> Result<u16, PayloadError> {
        let (ptype, tlen) = self.read_type_info()?;
        if ptype != PayloadType::Unsigned || tlen != TypeLength::Bit16 {
            return Err(PayloadError::InvalidType);
        }
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// Read an unsigned 32-bit integer
    pub fn read_u32(&mut self) -> Result<u32, PayloadError> {
        let (ptype, tlen) = self.read_type_info()?;
        if ptype != PayloadType::Unsigned || tlen != TypeLength::Bit32 {
            return Err(PayloadError::InvalidType);
        }
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read an unsigned 64-bit integer
    pub fn read_u64(&mut self) -> Result<u64, PayloadError> {
        let (ptype, tlen) = self.read_type_info()?;
        if ptype != PayloadType::Unsigned || tlen != TypeLength::Bit64 {
            return Err(PayloadError::InvalidType);
        }
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Read an unsigned 128-bit integer
    pub fn read_u128(&mut self) -> Result<u128, PayloadError> {
        let (ptype, tlen) = self.read_type_info()?;
        if ptype != PayloadType::Unsigned || tlen != TypeLength::Bit128 {
            return Err(PayloadError::InvalidType);
        }
        let bytes = self.read_bytes(16)?;
        Ok(u128::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11],
            bytes[12], bytes[13], bytes[14], bytes[15],
        ]))
    }

    /// Read a 32-bit float
    pub fn read_f32(&mut self) -> Result<f32, PayloadError> {
        let (ptype, tlen) = self.read_type_info()?;
        if ptype != PayloadType::Float || tlen != TypeLength::Bit32 {
            return Err(PayloadError::InvalidType);
        }
        let bytes = self.read_bytes(4)?;
        Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read a 64-bit float
    pub fn read_f64(&mut self) -> Result<f64, PayloadError> {
        let (ptype, tlen) = self.read_type_info()?;
        if ptype != PayloadType::Float || tlen != TypeLength::Bit64 {
            return Err(PayloadError::InvalidType);
        }
        let bytes = self.read_bytes(8)?;
        Ok(f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Read a string
    pub fn read_string(&mut self) -> Result<&'a str, PayloadError> {
        let (ptype, _) = self.read_type_info()?;
        if ptype != PayloadType::String {
            return Err(PayloadError::InvalidType);
        }

        // Read length (includes null terminator)
        let len_bytes = self.read_bytes(2)?;
        let len = u16::from_le_bytes([len_bytes[0], len_bytes[1]]) as usize;

        if len == 0 {
            return Err(PayloadError::InvalidData);
        }

        // Read string data (including null terminator)
        let string_data = self.read_bytes(len)?;

        // Verify null terminator
        if string_data[len - 1] != 0 {
            return Err(PayloadError::InvalidData);
        }

        // Convert to str (excluding null terminator)
        core::str::from_utf8(&string_data[..len - 1]).map_err(|_| PayloadError::InvalidData)
    }

    /// Read raw bytes
    pub fn read_raw(&mut self) -> Result<&'a [u8], PayloadError> {
        let (ptype, _) = self.read_type_info()?;
        if ptype != PayloadType::Raw {
            return Err(PayloadError::InvalidType);
        }

        // Read length (includes null terminator)
        let len_bytes = self.read_bytes(2)?;
        let len = u16::from_le_bytes([len_bytes[0], len_bytes[1]]) as usize;

        if len == 0 {
            return Err(PayloadError::InvalidData);
        }

        // Read raw data (including null terminator)
        let raw_data = self.read_bytes(len)?;

        // Verify null terminator
        if raw_data[len - 1] != 0 {
            return Err(PayloadError::InvalidData);
        }

        // Return data (excluding null terminator)
        Ok(&raw_data[..len - 1])
    }

    /// Peek at the next type info without consuming it
    pub fn peek_type_info(&self) -> Result<(PayloadType, TypeLength), PayloadError> {
        if self.position + 4 > self.data.len() {
            return Err(PayloadError::BufferTooSmall);
        }

        let bytes = &self.data[self.position..self.position + 4];
        let type_info = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let type_length = match type_info & 0x0F {
            0x00 => TypeLength::NotDefined,
            0x01 => TypeLength::Bit8,
            0x02 => TypeLength::Bit16,
            0x03 => TypeLength::Bit32,
            0x04 => TypeLength::Bit64,
            0x05 => TypeLength::Bit128,
            _ => return Err(PayloadError::InvalidType),
        };

        let payload_type = PayloadType::parse(type_info).ok_or(PayloadError::InvalidType)?;

        Ok((payload_type, type_length))
    }

    /// Parse the next argument automatically based on its type info
    /// This is the primary method for parsing unknown payload types from incoming packets
    pub fn read_next(&mut self) -> Result<DltValue<'a>, PayloadError> {
        let (ptype, tlen) = self.peek_type_info()?;
        
        match ptype {
            PayloadType::Bool => {
                let val = self.read_bool()?;
                Ok(DltValue::Bool(val))
            }
            PayloadType::Signed => {
                match tlen {
                    TypeLength::Bit8 => Ok(DltValue::I8(self.read_i8()?)),
                    TypeLength::Bit16 => Ok(DltValue::I16(self.read_i16()?)),
                    TypeLength::Bit32 => Ok(DltValue::I32(self.read_i32()?)),
                    TypeLength::Bit64 => Ok(DltValue::I64(self.read_i64()?)),
                    _ => Err(PayloadError::UnsupportedLength),
                }
            }
            PayloadType::Unsigned => {
                match tlen {
                    TypeLength::Bit8 => Ok(DltValue::U8(self.read_u8()?)),
                    TypeLength::Bit16 => Ok(DltValue::U16(self.read_u16()?)),
                    TypeLength::Bit32 => Ok(DltValue::U32(self.read_u32()?)),
                    TypeLength::Bit64 => Ok(DltValue::U64(self.read_u64()?)),
                    TypeLength::Bit128 => Ok(DltValue::U128(self.read_u128()?)),
                    _ => Err(PayloadError::UnsupportedLength),
                }
            }
            PayloadType::Float => {
                match tlen {
                    TypeLength::Bit32 => Ok(DltValue::F32(self.read_f32()?)),
                    TypeLength::Bit64 => Ok(DltValue::F64(self.read_f64()?)),
                    _ => Err(PayloadError::UnsupportedLength),
                }
            }
            PayloadType::String => {
                let val = self.read_string()?;
                Ok(DltValue::String(val))
            }
            PayloadType::Raw => {
                let val = self.read_raw()?;
                Ok(DltValue::Raw(val))
            }
            _ => Err(PayloadError::InvalidType),
        }
    }

    /// Parse all remaining arguments into a collection
    /// Returns a fixed-size array to avoid heap allocation
    /// The caller must provide a buffer large enough to hold all arguments
    pub fn read_all_args<'b>(
        &mut self,
        buffer: &'b mut [Option<DltValue<'a>>],
    ) -> Result<usize, PayloadError> {
        let mut count = 0;
        
        while !self.is_empty() && count < buffer.len() {
            buffer[count] = Some(self.read_next()?);
            count += 1;
        }
        
        Ok(count)
    }

    /// Skip the next argument without parsing it
    pub fn skip_argument(&mut self) -> Result<(), PayloadError> {
        let (ptype, tlen) = self.read_type_info()?;

        match ptype {
            PayloadType::Bool | PayloadType::Signed | PayloadType::Unsigned | PayloadType::Float => {
                let size = tlen.to_bytes();
                self.read_bytes(size)?;
            }
            PayloadType::String | PayloadType::Raw => {
                // Read length field
                let len_bytes = self.read_bytes(2)?;
                let len = u16::from_le_bytes([len_bytes[0], len_bytes[1]]) as usize;
                // Skip data
                self.read_bytes(len)?;
            }
            _ => return Err(PayloadError::InvalidType),
        }

        Ok(())
    }

    /// Reset parser position to the beginning
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Get current position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Seek to a specific position
    pub fn seek(&mut self, position: usize) -> Result<(), PayloadError> {
        if position > self.data.len() {
            return Err(PayloadError::BufferTooSmall);
        }
        self.position = position;
        Ok(())
    }
}
