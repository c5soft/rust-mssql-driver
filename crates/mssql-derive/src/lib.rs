//! # mssql-derive
//!
//! Procedural macros for SQL Server row mapping.
//!
//! This crate provides derive macros for automatically implementing
//! row-to-struct mapping for query results.
//!
//! ## Example
//!
//! ```rust,ignore
//! use mssql_derive::FromRow;
//!
//! #[derive(FromRow)]
//! struct User {
//!     id: i32,
//!     #[mssql(rename = "user_name")]
//!     name: String,
//!     email: Option<String>,
//! }
//! ```

#![warn(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Derive macro for implementing `FromRow` trait.
///
/// This macro generates code to convert a database row into a struct.
///
/// ## Attributes
///
/// - `#[mssql(rename = "column_name")]` - Map field to a different column name
/// - `#[mssql(skip)]` - Skip this field (must have a Default implementation)
///
/// ## Example
///
/// ```rust,ignore
/// #[derive(FromRow)]
/// struct User {
///     id: i32,
///     #[mssql(rename = "user_name")]
///     name: String,
///     #[mssql(skip)]
///     computed: String,
/// }
/// ```
#[proc_macro_derive(FromRow, attributes(mssql))]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Placeholder implementation
    // Real implementation would:
    // 1. Parse struct fields
    // 2. Handle #[mssql] attributes
    // 3. Generate FromRow implementation

    let expanded = quote! {
        // Placeholder: actual implementation would be generated here
        impl #name {
            /// Placeholder for FromRow implementation
            pub fn __from_row_placeholder() {}
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for implementing `ToParams` trait.
///
/// This macro generates code to convert a struct into query parameters.
///
/// ## Example
///
/// ```rust,ignore
/// #[derive(ToParams)]
/// struct NewUser {
///     name: String,
///     email: String,
/// }
///
/// let user = NewUser { name: "Alice".into(), email: "alice@example.com".into() };
/// client.execute("INSERT INTO users (name, email) VALUES (@name, @email)", &user).await?;
/// ```
#[proc_macro_derive(ToParams, attributes(mssql))]
pub fn derive_to_params(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let expanded = quote! {
        // Placeholder: actual implementation would be generated here
        impl #name {
            /// Placeholder for ToParams implementation
            pub fn __to_params_placeholder() {}
        }
    };

    TokenStream::from(expanded)
}
