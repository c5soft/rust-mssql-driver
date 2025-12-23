//! Protocol-level error types.

use thiserror::Error;

/// Errors that can occur during TDS protocol parsing or encoding.
#[derive(Debug, Error)]
pub enum ProtocolError {
    /// Packet data is truncated or incomplete.
    #[error("incomplete packet: expected {expected} bytes, got {actual}")]
    IncompletePacket {
        /// Expected number of bytes.
        expected: usize,
        /// Actual number of bytes available.
        actual: usize,
    },

    /// Invalid packet type value.
    #[error("invalid packet type: {0:#x}")]
    InvalidPacketType(u8),

    /// Invalid token type value.
    #[error("invalid token type: {0:#x}")]
    InvalidTokenType(u8),

    /// Invalid data type value.
    #[error("invalid data type: {0:#x}")]
    InvalidDataType(u8),

    /// Invalid prelogin option.
    #[error("invalid prelogin option: {0:#x}")]
    InvalidPreloginOption(u8),

    /// Invalid TDS version.
    #[error("invalid TDS version: {0:#x}")]
    InvalidTdsVersion(u32),

    /// String encoding error.
    #[error("string encoding error: {0}")]
    StringEncoding(
        #[cfg(feature = "std")] String,
        #[cfg(not(feature = "std"))] &'static str,
    ),

    /// Packet length exceeds maximum allowed.
    #[error("packet too large: {length} bytes (max {max})")]
    PacketTooLarge {
        /// Actual packet length.
        length: usize,
        /// Maximum allowed length.
        max: usize,
    },

    /// Invalid packet status flags.
    #[error("invalid packet status: {0:#x}")]
    InvalidPacketStatus(u8),

    /// Buffer overflow during encoding.
    #[error("buffer overflow: needed {needed} bytes, capacity {capacity}")]
    BufferOverflow {
        /// Bytes needed.
        needed: usize,
        /// Buffer capacity.
        capacity: usize,
    },

    /// Unexpected end of stream.
    #[error("unexpected end of stream")]
    UnexpectedEof,

    /// Protocol version mismatch.
    #[error("unsupported protocol version: {0}")]
    UnsupportedVersion(u32),

    /// Invalid field value in a protocol structure.
    #[error("invalid {field} value: {value}")]
    InvalidField {
        /// Field name.
        field: &'static str,
        /// Invalid value.
        value: u32,
    },
}
