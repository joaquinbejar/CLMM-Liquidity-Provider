//! Repository implementations for database persistence.
//!
//! This module provides repository patterns for storing and retrieving
//! simulation data, pool configurations, and price history.

mod pool_repository;
mod price_repository;
mod simulation_repository;

pub use pool_repository::{PoolRecord, PoolRepository};
pub use price_repository::{PriceRecord, PriceRepository};
pub use simulation_repository::{
    OptimizationRecord, SimulationRecord, SimulationRepository, SimulationResultRecord,
};

use sqlx::PgPool;
use std::sync::Arc;

/// Database connection wrapper for repositories.
#[derive(Clone)]
pub struct Database {
    pool: Arc<PgPool>,
}

impl Database {
    /// Creates a new Database wrapper from a connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }

    /// Creates a new database connection from a connection string.
    ///
    /// # Arguments
    /// * `database_url` - PostgreSQL connection string
    ///
    /// # Errors
    /// Returns an error if the connection fails.
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self::new(pool))
    }

    /// Returns a reference to the connection pool.
    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Creates a PoolRepository instance.
    #[must_use]
    pub fn pools(&self) -> PoolRepository {
        PoolRepository::new(self.pool.clone())
    }

    /// Creates a SimulationRepository instance.
    #[must_use]
    pub fn simulations(&self) -> SimulationRepository {
        SimulationRepository::new(self.pool.clone())
    }

    /// Creates a PriceRepository instance.
    #[must_use]
    pub fn prices(&self) -> PriceRepository {
        PriceRepository::new(self.pool.clone())
    }

    /// Runs database migrations.
    ///
    /// # Errors
    /// Returns an error if migrations fail.
    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::query(include_str!("../../migrations/001_initial_schema.sql"))
            .execute(self.pool.as_ref())
            .await?;
        Ok(())
    }
}
