//! Emergency controls and circuit breaker.
//!
//! Provides safety mechanisms for automated trading:
//! - Circuit breaker for consecutive failures
//! - Emergency position exit
//! - Loss threshold protection

mod circuit_breaker;
mod emergency_exit;

pub use circuit_breaker::*;
pub use emergency_exit::*;
