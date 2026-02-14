//! # DLT Service/Control Message Parser (Re-export)
//!
//! This module re-exports the service parser from `parse_service` for backward compatibility.
//! New code should use `parse_service` directly.

pub use crate::r19_11::parse_service::*;
