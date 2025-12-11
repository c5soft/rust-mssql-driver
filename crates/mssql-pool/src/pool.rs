//! Connection pool implementation.
//!
//! This module provides a purpose-built connection pool for SQL Server
//! with SQL Server-specific lifecycle management including `sp_reset_connection`.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use parking_lot::Mutex;

use crate::config::PoolConfig;
use crate::error::PoolError;
use crate::lifecycle::ConnectionMetadata;

/// A connection pool for SQL Server.
///
/// The pool manages a set of database connections, providing automatic
/// connection reuse, health checking, and lifecycle management.
///
/// # Features
///
/// - `sp_reset_connection` execution on connection return
/// - Health checks via `SELECT 1`
/// - Configurable min/max pool sizes
/// - Connection timeout and idle timeout
/// - Automatic reconnection on transient failures
///
/// # Example
///
/// ```rust,ignore
/// use mssql_driver_pool::{Pool, PoolConfig};
/// use mssql_client::Config;
///
/// let pool_config = PoolConfig::new()
///     .min_connections(5)
///     .max_connections(20);
///
/// let pool = Pool::builder()
///     .connection_config(client_config)
///     .pool_config(pool_config)
///     .build()
///     .await?;
///
/// let conn = pool.get().await?;
/// // Use connection...
/// ```
pub struct Pool {
    config: PoolConfig,
    inner: Arc<PoolInner>,
}

struct PoolInner {
    /// Pool configuration.
    #[allow(dead_code)] // Will be used once pool implementation is complete
    config: PoolConfig,

    /// Whether the pool is closed.
    closed: AtomicBool,

    /// Counter for generating connection IDs.
    #[allow(dead_code)] // Used when connection creation is implemented
    next_connection_id: AtomicU64,

    /// When the pool was created.
    created_at: Instant,

    /// Pool metrics.
    metrics: Mutex<PoolMetricsInner>,
}

/// Internal metrics tracking.
#[derive(Debug, Default)]
struct PoolMetricsInner {
    /// Total connections created.
    connections_created: u64,
    /// Total connections closed.
    connections_closed: u64,
    /// Total successful checkouts.
    checkouts_successful: u64,
    /// Total failed checkouts (timeouts, errors).
    checkouts_failed: u64,
    /// Total health checks performed.
    health_checks_performed: u64,
    /// Total health check failures.
    health_checks_failed: u64,
    /// Total resets performed.
    resets_performed: u64,
    /// Total reset failures.
    resets_failed: u64,
}

impl Pool {
    /// Create a new pool builder.
    ///
    /// Use the builder to configure the pool before creating it.
    #[must_use]
    pub fn builder() -> PoolBuilder {
        PoolBuilder::new()
    }

    /// Create a new pool with the given configuration.
    ///
    /// For more control over pool creation, use [`Pool::builder()`].
    pub async fn new(config: PoolConfig) -> Result<Self, PoolError> {
        config.validate()?;

        let inner = Arc::new(PoolInner {
            config: config.clone(),
            closed: AtomicBool::new(false),
            next_connection_id: AtomicU64::new(1),
            created_at: Instant::now(),
            metrics: Mutex::new(PoolMetricsInner::default()),
        });

        tracing::info!(
            min = config.min_connections,
            max = config.max_connections,
            "connection pool created"
        );

        Ok(Self { config, inner })
    }

    /// Get a connection from the pool.
    ///
    /// This will either return an existing idle connection or create a new one
    /// if the pool is not at capacity. If all connections are in use and the
    /// pool is at capacity, this will wait until a connection becomes available
    /// or the timeout is reached.
    pub async fn get(&self) -> Result<PooledConnection, PoolError> {
        if self.inner.closed.load(Ordering::Acquire) {
            return Err(PoolError::PoolClosed);
        }

        tracing::trace!("acquiring connection from pool");

        // Placeholder: actual connection acquisition logic
        // Would involve:
        // 1. Try to get idle connection
        // 2. If none, try to create new (if under max)
        // 3. If at max, wait with timeout

        todo!("Pool::get() - connection acquisition not yet implemented")
    }

    /// Try to get a connection without waiting.
    ///
    /// Returns `None` if no connections are immediately available.
    pub fn try_get(&self) -> Result<Option<PooledConnection>, PoolError> {
        if self.inner.closed.load(Ordering::Acquire) {
            return Err(PoolError::PoolClosed);
        }

        // Placeholder: actual non-blocking acquisition
        Ok(None)
    }

    /// Get the current pool status.
    #[must_use]
    pub fn status(&self) -> PoolStatus {
        PoolStatus {
            available: 0,
            in_use: 0,
            total: 0,
            max: self.config.max_connections,
        }
    }

    /// Get pool metrics.
    #[must_use]
    pub fn metrics(&self) -> PoolMetrics {
        let inner = self.inner.metrics.lock();
        PoolMetrics {
            connections_created: inner.connections_created,
            connections_closed: inner.connections_closed,
            checkouts_successful: inner.checkouts_successful,
            checkouts_failed: inner.checkouts_failed,
            health_checks_performed: inner.health_checks_performed,
            health_checks_failed: inner.health_checks_failed,
            resets_performed: inner.resets_performed,
            resets_failed: inner.resets_failed,
            uptime: self.inner.created_at.elapsed(),
        }
    }

    /// Close the pool, dropping all connections.
    pub async fn close(&self) {
        self.inner.closed.store(true, Ordering::Release);
        tracing::info!("connection pool closed");
    }

    /// Check if the pool is closed.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.inner.closed.load(Ordering::Acquire)
    }

    /// Get the pool configuration.
    #[must_use]
    pub fn config(&self) -> &PoolConfig {
        &self.config
    }

    /// Generate a new unique connection ID.
    #[allow(dead_code)] // Used when connection creation is implemented
    fn next_connection_id(&self) -> u64 {
        self.inner.next_connection_id.fetch_add(1, Ordering::Relaxed)
    }
}

/// Builder for creating a connection pool.
///
/// # Example
///
/// ```rust,ignore
/// let pool = Pool::builder()
///     .pool_config(pool_config)
///     .build()
///     .await?;
/// ```
pub struct PoolBuilder {
    pool_config: PoolConfig,
}

impl PoolBuilder {
    /// Create a new pool builder with default settings.
    pub fn new() -> Self {
        Self {
            pool_config: PoolConfig::default(),
        }
    }

    /// Set the pool configuration.
    #[must_use]
    pub fn pool_config(mut self, config: PoolConfig) -> Self {
        self.pool_config = config;
        self
    }

    /// Set the minimum number of connections.
    #[must_use]
    pub fn min_connections(mut self, count: u32) -> Self {
        self.pool_config.min_connections = count;
        self
    }

    /// Set the maximum number of connections.
    #[must_use]
    pub fn max_connections(mut self, count: u32) -> Self {
        self.pool_config.max_connections = count;
        self
    }

    /// Set the connection acquisition timeout.
    #[must_use]
    pub fn connection_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.pool_config.connection_timeout = timeout;
        self
    }

    /// Set the idle connection timeout.
    #[must_use]
    pub fn idle_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.pool_config.idle_timeout = timeout;
        self
    }

    /// Enable or disable `sp_reset_connection` on return.
    #[must_use]
    pub fn sp_reset_connection(mut self, enabled: bool) -> Self {
        self.pool_config.sp_reset_connection = enabled;
        self
    }

    /// Build the pool.
    pub async fn build(self) -> Result<Pool, PoolError> {
        Pool::new(self.pool_config).await
    }
}

impl Default for PoolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Status information about the pool.
#[derive(Debug, Clone, Copy)]
pub struct PoolStatus {
    /// Number of idle connections available.
    pub available: u32,
    /// Number of connections currently in use.
    pub in_use: u32,
    /// Total number of connections.
    pub total: u32,
    /// Maximum allowed connections.
    pub max: u32,
}

impl PoolStatus {
    /// Calculate the utilization percentage.
    #[must_use]
    pub fn utilization(&self) -> f64 {
        if self.max == 0 {
            return 0.0;
        }
        (self.in_use as f64 / self.max as f64) * 100.0
    }

    /// Check if the pool is at capacity.
    #[must_use]
    pub fn is_at_capacity(&self) -> bool {
        self.total >= self.max
    }
}

/// Metrics collected from the pool.
#[derive(Debug, Clone)]
pub struct PoolMetrics {
    /// Total connections created since pool start.
    pub connections_created: u64,
    /// Total connections closed since pool start.
    pub connections_closed: u64,
    /// Successful connection checkouts.
    pub checkouts_successful: u64,
    /// Failed connection checkouts (timeouts, pool closed, etc.).
    pub checkouts_failed: u64,
    /// Health checks performed.
    pub health_checks_performed: u64,
    /// Health checks that failed.
    pub health_checks_failed: u64,
    /// Connection resets performed.
    pub resets_performed: u64,
    /// Connection resets that failed.
    pub resets_failed: u64,
    /// Time since pool creation.
    pub uptime: std::time::Duration,
}

impl PoolMetrics {
    /// Calculate checkout success rate (0.0 to 1.0).
    #[must_use]
    pub fn checkout_success_rate(&self) -> f64 {
        let total = self.checkouts_successful + self.checkouts_failed;
        if total == 0 {
            return 1.0;
        }
        self.checkouts_successful as f64 / total as f64
    }

    /// Calculate health check success rate (0.0 to 1.0).
    #[must_use]
    pub fn health_check_success_rate(&self) -> f64 {
        if self.health_checks_performed == 0 {
            return 1.0;
        }
        let successful = self.health_checks_performed - self.health_checks_failed;
        successful as f64 / self.health_checks_performed as f64
    }
}

/// A connection retrieved from the pool.
///
/// When dropped, the connection is automatically returned to the pool.
/// Use [`detach()`](PooledConnection::detach) to prevent automatic return.
pub struct PooledConnection {
    /// Connection metadata.
    #[allow(dead_code)] // Will be used once pool implementation is complete
    metadata: ConnectionMetadata,
    /// Reference to the pool for returning the connection.
    #[allow(dead_code)] // Will be used once pool implementation is complete
    pool: Arc<PoolInner>,
}

impl PooledConnection {
    /// Create a new pooled connection.
    #[allow(dead_code)] // Used when connection acquisition is implemented
    fn new(metadata: ConnectionMetadata, pool: Arc<PoolInner>) -> Self {
        Self { metadata, pool }
    }

    /// Get the connection metadata.
    #[must_use]
    pub fn metadata(&self) -> &ConnectionMetadata {
        &self.metadata
    }

    /// Detach the connection from the pool.
    ///
    /// The connection will not be returned to the pool when dropped.
    /// This is useful when you want to keep the connection beyond the
    /// normal pool lifecycle.
    pub fn detach(self) {
        // Prevent returning to pool by forgetting the wrapper
        std::mem::forget(self);
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        // Return connection to pool
        // Would involve:
        // 1. Run sp_reset_connection if configured
        // 2. Return to idle queue
        tracing::trace!(
            connection_id = self.metadata.id,
            "returning connection to pool"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_status_utilization() {
        let status = PoolStatus {
            available: 5,
            in_use: 5,
            total: 10,
            max: 20,
        };
        assert!((status.utilization() - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pool_status_at_capacity() {
        let status = PoolStatus {
            available: 0,
            in_use: 10,
            total: 10,
            max: 10,
        };
        assert!(status.is_at_capacity());

        let status2 = PoolStatus {
            available: 5,
            in_use: 5,
            total: 10,
            max: 20,
        };
        assert!(!status2.is_at_capacity());
    }

    #[test]
    fn test_pool_metrics_success_rates() {
        let metrics = PoolMetrics {
            connections_created: 10,
            connections_closed: 2,
            checkouts_successful: 90,
            checkouts_failed: 10,
            health_checks_performed: 100,
            health_checks_failed: 5,
            resets_performed: 80,
            resets_failed: 2,
            uptime: std::time::Duration::from_secs(3600),
        };

        assert!((metrics.checkout_success_rate() - 0.9).abs() < f64::EPSILON);
        assert!((metrics.health_check_success_rate() - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_builder_default() {
        let builder = PoolBuilder::new();
        assert_eq!(builder.pool_config.min_connections, 1);
        assert_eq!(builder.pool_config.max_connections, 10);
    }

    #[test]
    fn test_builder_fluent() {
        let builder = Pool::builder()
            .min_connections(5)
            .max_connections(50)
            .sp_reset_connection(false);

        assert_eq!(builder.pool_config.min_connections, 5);
        assert_eq!(builder.pool_config.max_connections, 50);
        assert!(!builder.pool_config.sp_reset_connection);
    }
}
