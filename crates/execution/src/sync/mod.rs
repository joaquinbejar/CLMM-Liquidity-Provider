//! State synchronization with on-chain data.
//!
//! Provides real-time synchronization via:
//! - WebSocket account subscriptions
//! - Slot tracking
//! - State reconciliation

mod account_listener;
mod reconciler;

pub use account_listener::*;
pub use reconciler::*;
