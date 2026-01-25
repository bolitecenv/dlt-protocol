#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DltError {
    BufferTooSmall,
    InvalidParameter,
}
