//! Health-check types for the event bus.

/// Health status of the event bus connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusHealth {
    Connected,
    Disconnected,
}
