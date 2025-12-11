//! # mssql-codec
//!
//! Async framing layer for TDS packet handling.
//!
//! This crate transforms raw byte streams into high-level TDS packets,
//! handling packet reassembly across TCP segment boundaries and packet
//! continuation for large messages.
//!
//! ## Features
//!
//! - Packet reassembly across TCP segments
//! - Packet continuation handling (large packets split across multiple TDS packets)
//! - IO splitting for cancellation safety
//! - Integration with tokio-util's codec framework

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod error;
pub mod framed;
pub mod packet_codec;

pub use error::CodecError;
pub use framed::PacketStream;
pub use packet_codec::TdsCodec;
