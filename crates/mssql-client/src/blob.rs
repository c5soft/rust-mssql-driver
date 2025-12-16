//! Streaming large object (LOB) reader for VARBINARY(MAX) and TEXT types.
//!
//! This module provides `BlobReader` for streaming large binary objects without
//! loading them entirely into memory. This is particularly useful for:
//!
//! - Files stored as VARBINARY(MAX) (multi-GB)
//! - Large TEXT/NTEXT columns
//! - XML documents stored as XML type
//!
//! ## Status
//!
//! **NOT YET IMPLEMENTED** - This is a future enhancement.
//!
//! The current implementation loads all LOB data into memory via `Arc<Bytes>`.
//! For most use cases (LOBs < 100MB), this is acceptable.
//!
//! ## Future API
//!
//! When implemented, usage would look like:
//!
//! ```rust,ignore
//! use mssql_client::blob::BlobReader;
//! use tokio::io::AsyncReadExt;
//!
//! let stream = client.query("SELECT large_file FROM documents WHERE id = @p1", &[&id]).await?;
//!
//! if let Some(row) = stream.next().await? {
//!     // Get a streaming reader for the BLOB column
//!     let mut blob: BlobReader = row.get_stream(0)?;
//!
//!     // Stream to file without loading entire BLOB into memory
//!     let mut file = tokio::fs::File::create("output.bin").await?;
//!     tokio::io::copy(&mut blob, &mut file).await?;
//! }
//! ```
//!
//! ## Implementation Notes
//!
//! Streaming LOBs requires:
//! 1. Partial row retrieval at the TDS protocol layer
//! 2. TEXTPTR/READTEXT for legacy TEXT types, or
//! 3. Chunked retrieval for VARBINARY(MAX) using offset queries
//! 4. Connection affinity (must use same connection for all chunks)
//!
//! The TDS protocol itself doesn't support true streaming; implementation
//! would use server-side cursors or chunked queries internally.

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, ReadBuf};

/// Streaming reader for large binary objects.
///
/// **NOT YET IMPLEMENTED** - Returns `Unimplemented` error on all operations.
///
/// See module documentation for the planned API.
pub struct BlobReader {
    // Future fields:
    // connection: Arc<Mutex<Connection>>,
    // column_index: usize,
    // total_length: Option<u64>,
    // bytes_read: u64,
    // buffer: BytesMut,
}

impl BlobReader {
    /// Create a new BlobReader.
    ///
    /// **NOT YET IMPLEMENTED**
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    /// Get the total length of the BLOB if known.
    ///
    /// Returns `None` if the length is unknown (streaming without length hint).
    #[must_use]
    pub fn len(&self) -> Option<u64> {
        None
    }

    /// Check if the BLOB is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len().is_some_and(|len| len == 0)
    }

    /// Get the number of bytes read so far.
    #[must_use]
    pub fn bytes_read(&self) -> u64 {
        0
    }
}

impl Default for BlobReader {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncRead for BlobReader {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Poll::Ready(Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "BlobReader not yet implemented - use Arc<Bytes> pattern for now",
        )))
    }
}

impl std::fmt::Debug for BlobReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlobReader")
            .field("status", &"not_implemented")
            .finish()
    }
}
