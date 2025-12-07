//! Position lifecycle tracking.
//!
//! Tracks the complete lifecycle of LP positions:
//! - Position opening
//! - Rebalancing events
//! - Fee collections
//! - Position closing

mod events;
mod tracker;

pub use events::*;
pub use tracker::*;
