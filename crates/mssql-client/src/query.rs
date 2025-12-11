//! Query builder and prepared statement support.

use mssql_types::ToSql;

/// A prepared query builder.
///
/// Queries can be built incrementally and reused with different parameters.
#[derive(Debug, Clone)]
pub struct Query {
    sql: String,
    // Placeholder for prepared statement handle and metadata
}

impl Query {
    /// Create a new query from SQL text.
    #[must_use]
    pub fn new(sql: impl Into<String>) -> Self {
        Self { sql: sql.into() }
    }

    /// Get the SQL text.
    #[must_use]
    pub fn sql(&self) -> &str {
        &self.sql
    }
}

/// Extension trait for building parameterized queries.
pub trait QueryExt {
    /// Add a parameter to the query.
    fn bind<T: ToSql>(self, value: &T) -> BoundQuery<'_>;
}

/// A query with bound parameters.
pub struct BoundQuery<'a> {
    sql: &'a str,
    params: Vec<&'a dyn ToSql>,
}

impl<'a> BoundQuery<'a> {
    /// Create a new bound query.
    pub fn new(sql: &'a str) -> Self {
        Self {
            sql,
            params: Vec::new(),
        }
    }

    /// Add another parameter.
    pub fn bind<T: ToSql>(mut self, value: &'a T) -> Self {
        self.params.push(value);
        self
    }

    /// Get the SQL text.
    #[must_use]
    pub fn sql(&self) -> &str {
        self.sql
    }

    /// Get the bound parameters.
    #[must_use]
    pub fn params(&self) -> &[&dyn ToSql] {
        &self.params
    }
}
