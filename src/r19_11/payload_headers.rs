enum MessageType {
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
}

#[derive(Debug)]
pub struct Message {
    message_type: MessageType,
    payload: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct MessageList {
    msg_list: Vec<Message>,
}

impl MessageList {
    fn parse_type(type_byte: u32) -> Option<MessageType> {
        if type_byte & 0x10 != 0 {
            return Some(MessageType::Bool);
        }
        if type_byte & 0x20 != 0 {
            return Some(MessageType::Signed);
        }
        if type_byte & 0x40 != 0 {
            return Some(MessageType::Unsigned);
        }
        if type_byte & 0x80 != 0 {
            return Some(MessageType::Float);
        }
        if type_byte & 0x100 != 0 {
            return Some(MessageType::Array);
        }
        if type_byte & 0x200 != 0 {
            return Some(MessageType::String);
        }
        if type_byte & 0x400 != 0 {
            return Some(MessageType::Raw);
        }
        if type_byte & 0x800 != 0 {
            return Some(MessageType::VariableInfo);
        }
        if type_byte & 0x1000 != 0 {
            return Some(MessageType::FixedPoint);
        }
        if type_byte & 0x2000 != 0 {
            return Some(MessageType::TraceInfo);
        }
        if type_byte & 0x4000 != 0 {
            return Some(MessageType::Struct);
        }
        if type_byte & 0x8000 != 0 {
            return Some(MessageType::StringCoding);
        }
        None
    }
}
