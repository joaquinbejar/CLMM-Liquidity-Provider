//! Live execution engine and transaction management.
//!
//! This crate provides functionality for live position management:
//! - Position monitoring with real-time PnL tracking
//! - Alert system with configurable rules
//! - Wallet management for transaction signing
//! - Transaction building and lifecycle management
//! - Automated strategy execution
//! - Emergency controls and circuit breaker
//! - Position lifecycle tracking
//! - State synchronization

/// Prelude module for convenient imports.
pub mod prelude;

/// Alert system.
pub mod alerts;
/// Emergency controls and circuit breaker.
pub mod emergency;
/// Position lifecycle tracking.
pub mod lifecycle;
/// Position monitoring.
pub mod monitor;
/// Scheduler for strategy timing.
pub mod scheduler;
/// Strategy execution.
pub mod strategy;
/// State synchronization.
pub mod sync;
/// Transaction building and sending.
pub mod transaction;
/// Wallet management.
pub mod wallet;
