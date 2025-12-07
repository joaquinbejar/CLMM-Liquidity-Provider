//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types from the crate.
//!
//! # Example
//!
//! ```rust
//! use clmm_lp_simulation::prelude::*;
//! ```

// Engine
pub use crate::engine::SimulationEngine;

// Liquidity models
pub use crate::liquidity::{ConstantLiquidity, LiquidityModel};

// Monte Carlo
pub use crate::monte_carlo::{AggregateResult, MonteCarloRunner};

// Position tracking
pub use crate::position_tracker::{PositionSnapshot, PositionTracker, TrackerSummary};

// Price path generators
pub use crate::price_path::{
    DeterministicPricePath, GeometricBrownianMotion, HistoricalPricePath, PricePathGenerator,
};

// Strategies
pub use crate::strategies::{
    ILLimitStrategy, PeriodicRebalance, RebalanceAction, RebalanceReason, RebalanceStrategy,
    StaticRange, StrategyContext, ThresholdRebalance,
};

// Volume models
pub use crate::volume::{ConstantVolume, VolumeModel};
