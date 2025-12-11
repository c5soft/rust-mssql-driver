//! Row representation for query results.

use mssql_types::{FromSql, SqlValue, TypeError};

/// A row from a query result.
#[derive(Debug, Clone)]
pub struct Row {
    columns: Vec<Column>,
    values: Vec<SqlValue>,
}

/// Column metadata.
#[derive(Debug, Clone)]
pub struct Column {
    /// Column name.
    pub name: String,
    /// Column index.
    pub index: usize,
    /// SQL type name.
    pub type_name: String,
    /// Whether the column is nullable.
    pub nullable: bool,
}

impl Row {
    /// Create a new row from columns and values.
    #[allow(dead_code)] // Will be used once query execution is implemented
    pub(crate) fn new(columns: Vec<Column>, values: Vec<SqlValue>) -> Self {
        Self { columns, values }
    }

    /// Get a value by column index.
    pub fn get<T: FromSql>(&self, index: usize) -> Result<T, TypeError> {
        self.values
            .get(index)
            .ok_or_else(|| TypeError::TypeMismatch {
                expected: "valid column index",
                actual: format!("index {index} out of bounds"),
            })
            .and_then(T::from_sql)
    }

    /// Get a value by column name.
    pub fn get_by_name<T: FromSql>(&self, name: &str) -> Result<T, TypeError> {
        let index = self
            .columns
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| TypeError::TypeMismatch {
                expected: "valid column name",
                actual: format!("column '{name}' not found"),
            })?;

        self.get(index)
    }

    /// Try to get a value by column index, returning None if NULL or not found.
    pub fn try_get<T: FromSql>(&self, index: usize) -> Option<T> {
        self.values
            .get(index)
            .and_then(|v| T::from_sql_nullable(v).ok().flatten())
    }

    /// Try to get a value by column name, returning None if NULL or not found.
    pub fn try_get_by_name<T: FromSql>(&self, name: &str) -> Option<T> {
        let index = self
            .columns
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(name))?;

        self.try_get(index)
    }

    /// Get the raw SQL value by index.
    #[must_use]
    pub fn get_raw(&self, index: usize) -> Option<&SqlValue> {
        self.values.get(index)
    }

    /// Get the raw SQL value by column name.
    #[must_use]
    pub fn get_raw_by_name(&self, name: &str) -> Option<&SqlValue> {
        self.columns
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(name))
            .and_then(|i| self.values.get(i))
    }

    /// Get the number of columns in the row.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the row is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get the column metadata.
    #[must_use]
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    /// Iterate over (column, value) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&Column, &SqlValue)> {
        self.columns.iter().zip(self.values.iter())
    }
}

impl IntoIterator for Row {
    type Item = SqlValue;
    type IntoIter = std::vec::IntoIter<SqlValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<'a> IntoIterator for &'a Row {
    type Item = &'a SqlValue;
    type IntoIter = std::slice::Iter<'a, SqlValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}
