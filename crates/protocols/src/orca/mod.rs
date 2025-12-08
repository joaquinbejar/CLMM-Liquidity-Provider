//! Orca Whirlpool protocol adapter.
//!
//! This module provides functionality to interact with Orca Whirlpool pools:
//! - Read pool state
//! - Read position state
//! - Execute LP operations
//! - Calculate token amounts

/// Executor for on-chain operations.
pub mod executor;
/// Pool reader for on-chain state.
pub mod pool_reader;
/// Position reader for on-chain state.
pub mod position_reader;
/// Orca pool provider.
pub mod provider;
/// Orca whirlpool account structures.
pub mod whirlpool;
