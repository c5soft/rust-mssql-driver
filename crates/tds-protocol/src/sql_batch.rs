//! SQL batch request encoding.
//!
//! This module provides encoding for SQL batch requests (packet type 0x01).
//! A SQL batch is simply the SQL text encoded as UTF-16LE.

use bytes::{Bytes, BytesMut};

use crate::codec::write_utf16_string;

/// Encode a SQL batch request.
///
/// The SQL batch packet payload is simply the SQL text encoded as UTF-16LE.
/// This function returns the encoded payload (without the packet header).
///
/// # Example
///
/// ```
/// use tds_protocol::sql_batch::encode_sql_batch;
///
/// let sql = "SELECT * FROM users WHERE id = 1";
/// let payload = encode_sql_batch(sql);
///
/// // Payload is UTF-16LE encoded SQL
/// assert!(!payload.is_empty());
/// ```
#[must_use]
pub fn encode_sql_batch(sql: &str) -> Bytes {
    let mut buf = BytesMut::with_capacity(sql.len() * 2);
    write_utf16_string(&mut buf, sql);
    buf.freeze()
}

/// SQL batch builder for more complex batches.
///
/// This can be used to build batches with multiple statements
/// or to add headers for specific features.
#[derive(Debug, Clone)]
pub struct SqlBatch {
    sql: String,
}

impl SqlBatch {
    /// Create a new SQL batch.
    #[must_use]
    pub fn new(sql: impl Into<String>) -> Self {
        Self { sql: sql.into() }
    }

    /// Get the SQL text.
    #[must_use]
    pub fn sql(&self) -> &str {
        &self.sql
    }

    /// Encode the SQL batch to bytes.
    #[must_use]
    pub fn encode(&self) -> Bytes {
        encode_sql_batch(&self.sql)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_sql_batch() {
        let sql = "SELECT 1";
        let payload = encode_sql_batch(sql);

        // UTF-16LE encoded: 8 chars * 2 bytes = 16 bytes
        assert_eq!(payload.len(), 16);

        // Verify UTF-16LE encoding
        // 'S' = 0x53, 'E' = 0x45, 'L' = 0x4C, etc.
        assert_eq!(payload[0], b'S');
        assert_eq!(payload[1], 0);
        assert_eq!(payload[2], b'E');
        assert_eq!(payload[3], 0);
    }

    #[test]
    fn test_sql_batch_builder() {
        let batch = SqlBatch::new("SELECT @@VERSION");
        assert_eq!(batch.sql(), "SELECT @@VERSION");

        let payload = batch.encode();
        assert!(!payload.is_empty());
    }

    #[test]
    fn test_empty_batch() {
        let payload = encode_sql_batch("");
        assert!(payload.is_empty());
    }
}
