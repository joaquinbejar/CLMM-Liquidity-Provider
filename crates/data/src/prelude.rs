//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types from the crate.
//!
//! # Example
//!
//! ```rust
//! use clmm_lp_data::prelude::*;
//! ```

// Traits
pub use crate::MarketDataProvider;

// Providers
pub use crate::providers::csv_provider::write_candles_to_csv;
pub use crate::providers::{BirdeyeProvider, CsvProvider, MockMarketDataProvider};

// Database repositories
pub use crate::repositories::{
    Database, OptimizationRecord, PoolRecord, PoolRepository, PriceRecord, PriceRepository,
    SimulationRecord, SimulationRepository, SimulationResultRecord,
};
