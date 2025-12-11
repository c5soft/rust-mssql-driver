//! Connection state types for type-state pattern.
//!
//! The type-state pattern ensures at compile time that certain operations
//! can only be performed when the connection is in the appropriate state.

use std::marker::PhantomData;

/// Marker trait for connection states.
pub trait ConnectionState: private::Sealed {}

/// Connection is not yet established.
pub struct Disconnected;

/// Connection is established and ready for queries.
pub struct Ready;

/// Connection is in a transaction.
pub struct InTransaction;

impl ConnectionState for Disconnected {}
impl ConnectionState for Ready {}
impl ConnectionState for InTransaction {}

mod private {
    pub trait Sealed {}
    impl Sealed for super::Disconnected {}
    impl Sealed for super::Ready {}
    impl Sealed for super::InTransaction {}
}

/// Type-level state transition marker.
///
/// This is used internally to track state transitions at compile time.
#[derive(Debug)]
pub struct StateMarker<S: ConnectionState> {
    _state: PhantomData<S>,
}

impl<S: ConnectionState> StateMarker<S> {
    pub(crate) fn new() -> Self {
        Self {
            _state: PhantomData,
        }
    }
}

impl<S: ConnectionState> Default for StateMarker<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: ConnectionState> Clone for StateMarker<S> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<S: ConnectionState> Copy for StateMarker<S> {}
