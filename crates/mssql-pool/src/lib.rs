//! # mssql-driver-pool
//!
//! Purpose-built connection pool for SQL Server with lifecycle management.
//!
//! Unlike generic connection pools, this implementation understands SQL Server
//! specifics like `sp_reset_connection` for proper connection state cleanup.
//!
//! ## Features
//!
//! - `sp_reset_connection` execution on connection return
//! - Health checks via `SELECT 1`
//! - Configurable min/max pool sizes
//! - Connection timeout and idle timeout
//! - Automatic reconnection on transient failures
//! - Per-connection prepared statement cache management
//! - Comprehensive metrics for observability
//!
//! ## Example
//!
//! ```rust,ignore
//! use mssql_driver_pool::{Pool, PoolConfig};
//! use std::time::Duration;
//!
//! // Using the builder pattern
//! let pool = Pool::builder()
//!     .min_connections(5)
//!     .max_connections(20)
//!     .idle_timeout(Duration::from_secs(300))
//!     .sp_reset_connection(true)
//!     .build()
//!     .await?;
//!
//! // Or using PoolConfig directly
//! let config = PoolConfig::new()
//!     .min_connections(5)
//!     .max_connections(20);
//!
//! let pool = Pool::new(config).await?;
//!
//! // Get a connection from the pool
//! let conn = pool.get().await?;
//! // Use connection...
//! // Connection automatically returned to pool on drop
//!
//! // Check pool status
//! let status = pool.status();
//! println!("Pool utilization: {:.1}%", status.utilization());
//!
//! // Get metrics
//! let metrics = pool.metrics();
//! println!("Checkout success rate: {:.2}", metrics.checkout_success_rate());
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod config;
pub mod error;
pub mod lifecycle;
pub mod pool;

// Configuration
pub use config::PoolConfig;

// Error types
pub use error::PoolError;

// Pool types
pub use pool::{Pool, PoolBuilder, PoolMetrics, PoolStatus, PooledConnection};

// Lifecycle management
pub use lifecycle::{
    ConnectionLifecycle, ConnectionMetadata, ConnectionState, DynConnectionLifecycle,
    HealthCheckResult,
};
