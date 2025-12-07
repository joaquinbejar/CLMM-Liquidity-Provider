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

// Events
pub use crate::event::{EventData, EventLog, SimulationEvent, SimulationEventType};

// Liquidity models
pub use crate::liquidity::{ConstantLiquidity, LiquidityModel};

// Monte Carlo
pub use crate::monte_carlo::{AggregateResult, MonteCarloRunner};

// Position simulator
pub use crate::position_simulator::{PositionSimulationResult, simulate_position};

// Position tracking
pub use crate::position_tracker::{PositionSnapshot, PositionTracker, TrackerSummary};

// Price path generators
pub use crate::price_path::{
    DeterministicPricePath, GeometricBrownianMotion, HistoricalPricePath, PricePathGenerator,
};

// State management
pub use crate::state::{
    PoolState, PositionState, SimulationConfig, SimulationState, SimulationSummary,
};

// Strategies
pub use crate::strategies::{
    ILLimitStrategy, PeriodicRebalance, RebalanceAction, RebalanceReason, RebalanceStrategy,
    StaticRange, StrategyContext, ThresholdRebalance,
};

// Strategy simulator
pub use crate::strategy_simulator::{StrategySimulationResult, simulate_with_strategy};

// Volume models
pub use crate::volume::{ConstantVolume, VolumeModel};
