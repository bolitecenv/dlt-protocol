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
        let type_info = (type_length as u32) | payload_type.to_bit();
        self.write_bytes(&type_info.to_le_bytes())
    }

    /// Add a boolean value (8 bit)
    pub fn add_bool(&mut self, value: bool) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Bool, TypeLength::Bit8)?;
        self.write_bytes(&[value as u8])
    }

    /// Add a signed 8-bit integer
    pub fn add_i8(&mut self, value: i8) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Signed, TypeLength::Bit8)?;
        self.write_bytes(&value.to_le_bytes())
    }

    /// Add a signed 16-bit integer
    pub fn add_i16(&mut self, value: i16) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Signed, TypeLength::Bit16)?;
        self.write_bytes(&value.to_le_bytes())
    }

    /// Add a signed 32-bit integer
    pub fn add_i32(&mut self, value: i32) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Signed, TypeLength::Bit32)?;
        self.write_bytes(&value.to_le_bytes())
    }

    /// Add a signed 64-bit integer
    pub fn add_i64(&mut self, value: i64) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Signed, TypeLength::Bit64)?;
        self.write_bytes(&value.to_le_bytes())
    }

    /// Add an unsigned 8-bit integer
    pub fn add_u8(&mut self, value: u8) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Unsigned, TypeLength::Bit8)?;
        self.write_bytes(&value.to_le_bytes())
    }

    /// Add an unsigned 16-bit integer
    pub fn add_u16(&mut self, value: u16) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Unsigned, TypeLength::Bit16)?;
        self.write_bytes(&value.to_le_bytes())
    }

    /// Add an unsigned 32-bit integer
    pub fn add_u32(&mut self, value: u32) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Unsigned, TypeLength::Bit32)?;
        self.write_bytes(&value.to_le_bytes())
    }

    /// Add an unsigned 64-bit integer
    pub fn add_u64(&mut self, value: u64) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Unsigned, TypeLength::Bit64)?;
        self.write_bytes(&value.to_le_bytes())
    }

    /// Add a 32-bit float
    pub fn add_f32(&mut self, value: f32) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Float, TypeLength::Bit32)?;
        self.write_bytes(&value.to_le_bytes())
    }

    /// Add a 64-bit float
    pub fn add_f64(&mut self, value: f64) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Float, TypeLength::Bit64)?;
        self.write_bytes(&value.to_le_bytes())
    }

    /// Add a string
    /// For strings, the Type Length field is usually set to NotDefined (0x00)
    /// and the actual length is encoded in the payload
    pub fn add_string(&mut self, value: &str) -> Result<(), PayloadError> {
        // Type info with TYLE = 0 (not defined) for variable length strings
        self.write_type_info(PayloadType::String, TypeLength::NotDefined)?;

        // Write string length (2 bytes)
        let len = value.len() as u16;
        self.write_bytes(&len.to_le_bytes())?;

        // Write string data (null-terminated)
        self.write_bytes(value.as_bytes())?;
        self.write_bytes(&[0]) // null terminator
    }

    /// Add raw bytes
    pub fn add_raw(&mut self, data: &[u8]) -> Result<(), PayloadError> {
        // Type info with TYLE = 0 (not defined) for variable length raw data
        self.write_type_info(PayloadType::Raw, TypeLength::NotDefined)?;

        // Write data length (2 bytes)
        let len = data.len() as u16;
        self.write_bytes(&len.to_le_bytes())?;

        // Write raw data
        self.write_bytes(data)
    }

    /// Add a 128-bit value (generic)
    pub fn add_u128(&mut self, value: u128) -> Result<(), PayloadError> {
        self.write_type_info(PayloadType::Unsigned, TypeLength::Bit128)?;
        self.write_bytes(&value.to_le_bytes())
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

    /// Read a value based on the type info
    pub fn read_u32(&mut self) -> Result<u32, PayloadError> {
        let (ptype, tlen) = self.read_type_info()?;
        if ptype != PayloadType::Unsigned || tlen != TypeLength::Bit32 {
            return Err(PayloadError::InvalidType);
        }
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read a string
    pub fn read_string(&mut self) -> Result<&'a str, PayloadError> {
        let (ptype, _) = self.read_type_info()?;
        if ptype != PayloadType::String {
            return Err(PayloadError::InvalidType);
        }

        // Read length
        let len_bytes = self.read_bytes(2)?;
        let len = u16::from_le_bytes([len_bytes[0], len_bytes[1]]) as usize;

        // Read string data (including null terminator)
        let string_data = self.read_bytes(len + 1)?;

        // Convert to str (excluding null terminator)
        core::str::from_utf8(&string_data[..len]).map_err(|_| PayloadError::InvalidData)
    }
}
