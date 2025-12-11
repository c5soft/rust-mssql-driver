//! Transaction support.

/// Transaction isolation level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IsolationLevel {
    /// Read uncommitted (dirty reads allowed).
    ReadUncommitted,
    /// Read committed (default for SQL Server).
    #[default]
    ReadCommitted,
    /// Repeatable read.
    RepeatableRead,
    /// Serializable (highest isolation).
    Serializable,
    /// Snapshot isolation.
    Snapshot,
}

impl IsolationLevel {
    /// Get the SQL statement to set this isolation level.
    #[must_use]
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::ReadUncommitted => "SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED",
            Self::ReadCommitted => "SET TRANSACTION ISOLATION LEVEL READ COMMITTED",
            Self::RepeatableRead => "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ",
            Self::Serializable => "SET TRANSACTION ISOLATION LEVEL SERIALIZABLE",
            Self::Snapshot => "SET TRANSACTION ISOLATION LEVEL SNAPSHOT",
        }
    }
}

/// A database transaction.
///
/// This is a placeholder for a higher-level transaction abstraction
/// that could be used with a closure-based API.
pub struct Transaction<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Transaction<'a> {
    /// Get the isolation level of this transaction.
    #[must_use]
    pub fn isolation_level(&self) -> IsolationLevel {
        IsolationLevel::ReadCommitted
    }
}
