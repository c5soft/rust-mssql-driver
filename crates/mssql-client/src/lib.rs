//! # mssql-client
//!
//! High-level async SQL Server client with type-state connection management.
//!
//! This is the primary public API surface for the rust-mssql-driver project.
//! It provides a type-safe, ergonomic interface for working with SQL Server
//! databases.
//!
//! ## Features
//!
//! - **Type-state pattern**: Compile-time enforcement of connection states
//! - **Async/await**: Built on Tokio for efficient async I/O
//! - **Prepared statements**: Automatic caching with LRU eviction
//! - **Transactions**: Full transaction support with savepoints
//! - **Azure support**: Automatic routing and failover handling
//!
//! ## Example
//!
//! ```rust,ignore
//! use mssql_client::{Client, Config};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = Config::from_connection_string(
//!         "Server=localhost;Database=test;User Id=sa;Password=Password123;"
//!     )?;
//!
//!     let client = Client::connect(config).await?;
//!
//!     let rows = client
//!         .query("SELECT * FROM users WHERE id = @p1", &[&1])
//!         .await?;
//!
//!     for row in rows {
//!         let name: String = row.get("name")?;
//!         println!("User: {}", name);
//!     }
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod client;
pub mod config;
pub mod error;
pub mod query;
pub mod row;
pub mod state;
pub mod transaction;

// Re-export commonly used types
pub use client::Client;
pub use config::Config;
pub use error::Error;
pub use mssql_auth::Credentials;
pub use mssql_types::{FromSql, SqlValue, ToSql};
pub use query::Query;
pub use row::Row;
pub use state::{ConnectionState, Disconnected, InTransaction, Ready};
pub use transaction::Transaction;
