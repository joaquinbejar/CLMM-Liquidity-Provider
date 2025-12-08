//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types from the crate.
//!
//! # Example
//!
//! ```rust
//! use clmm_lp_protocols::prelude::*;
//! ```

// Traits
pub use crate::PoolFetcher;

// RPC provider
pub use crate::rpc::{CommitmentLevel, EndpointHealth, HealthChecker, RpcConfig, RpcProvider};

// Events
pub use crate::events::{
    ClosePositionEvent, CollectFeesEvent, EventFetcher, EventParser, FetchConfig, LiquidityEvent,
    OnChainPosition, OpenPositionEvent, Protocol, ProtocolEvent, SwapEvent, VolumeData,
    WhirlpoolInstruction,
};

// Orca
pub use crate::orca::executor::{
    DecreaseLiquidityParams, ExecutionResult, IncreaseLiquidityParams, OpenPositionParams,
    WhirlpoolExecutor,
};
pub use crate::orca::pool_reader::{
    WhirlpoolReader, WhirlpoolState, calculate_tick_range, price_to_tick, tick_to_price,
};
pub use crate::orca::position_reader::{PositionReader, WhirlpoolPosition};
pub use crate::orca::provider::OrcaPoolProvider;
pub use crate::orca::whirlpool::{Whirlpool, WhirlpoolParser};

// Solana client
pub use crate::solana_client::SolanaRpcAdapter;
