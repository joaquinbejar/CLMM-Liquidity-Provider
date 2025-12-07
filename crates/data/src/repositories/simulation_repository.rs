//! Simulation repository for backtest and optimization persistence.

use rust_decimal::Decimal;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

/// Database record for a simulation configuration.
#[derive(Debug, Clone)]
pub struct SimulationRecord {
    /// Unique identifier.
    pub id: Uuid,
    /// Associated pool ID.
    pub pool_id: Option<Uuid>,
    /// Strategy type (static, periodic, threshold).
    pub strategy_type: String,
    /// Strategy configuration as JSON.
    pub strategy_config: serde_json::Value,
    /// Start timestamp in seconds.
    pub start_timestamp: i64,
    /// End timestamp in seconds.
    pub end_timestamp: i64,
    /// Initial capital amount.
    pub initial_capital: Decimal,
    /// Entry price at simulation start.
    pub entry_price: Decimal,
    /// Lower price bound.
    pub lower_price: Decimal,
    /// Upper price bound.
    pub upper_price: Decimal,
    /// Fee rate as decimal.
    pub fee_rate: Decimal,
    /// Transaction cost per rebalance.
    pub tx_cost: Decimal,
    /// Record creation timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SimulationRecord {
    /// Creates a SimulationRecord from a database row.
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            pool_id: row.try_get("pool_id")?,
            strategy_type: row.try_get("strategy_type")?,
            strategy_config: row.try_get("strategy_config")?,
            start_timestamp: row.try_get("start_timestamp")?,
            end_timestamp: row.try_get("end_timestamp")?,
            initial_capital: row.try_get("initial_capital")?,
            entry_price: row.try_get("entry_price")?,
            lower_price: row.try_get("lower_price")?,
            upper_price: row.try_get("upper_price")?,
            fee_rate: row.try_get("fee_rate")?,
            tx_cost: row.try_get("tx_cost")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

/// Database record for simulation results.
#[derive(Debug, Clone)]
pub struct SimulationResultRecord {
    /// Unique identifier.
    pub id: Uuid,
    /// Associated simulation ID.
    pub simulation_id: Uuid,
    /// Final portfolio value.
    pub final_value: Decimal,
    /// Final profit/loss.
    pub final_pnl: Decimal,
    /// Total fees earned.
    pub total_fees: Decimal,
    /// Total impermanent loss.
    pub total_il: Decimal,
    /// Final IL as percentage.
    pub final_il_pct: Decimal,
    /// Time in range as percentage.
    pub time_in_range_pct: Decimal,
    /// Maximum drawdown as percentage.
    pub max_drawdown: Decimal,
    /// Number of rebalances executed.
    pub rebalance_count: i32,
    /// Total cost of rebalances.
    pub total_rebalance_cost: Decimal,
    /// HODL comparison value.
    pub hodl_value: Decimal,
    /// Difference vs HODL.
    pub vs_hodl: Decimal,
    /// Sharpe ratio if calculated.
    pub sharpe_ratio: Option<Decimal>,
    /// Final price at simulation end.
    pub final_price: Decimal,
    /// Record creation timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SimulationResultRecord {
    /// Creates a SimulationResultRecord from a database row.
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            simulation_id: row.try_get("simulation_id")?,
            final_value: row.try_get("final_value")?,
            final_pnl: row.try_get("final_pnl")?,
            total_fees: row.try_get("total_fees")?,
            total_il: row.try_get("total_il")?,
            final_il_pct: row.try_get("final_il_pct")?,
            time_in_range_pct: row.try_get("time_in_range_pct")?,
            max_drawdown: row.try_get("max_drawdown")?,
            rebalance_count: row.try_get("rebalance_count")?,
            total_rebalance_cost: row.try_get("total_rebalance_cost")?,
            hodl_value: row.try_get("hodl_value")?,
            vs_hodl: row.try_get("vs_hodl")?,
            sharpe_ratio: row.try_get("sharpe_ratio")?,
            final_price: row.try_get("final_price")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

/// Database record for optimization results.
#[derive(Debug, Clone)]
pub struct OptimizationRecord {
    /// Unique identifier.
    pub id: Uuid,
    /// Associated pool ID.
    pub pool_id: Option<Uuid>,
    /// Objective type (pnl, fees, sharpe).
    pub objective_type: String,
    /// Start timestamp in seconds.
    pub start_timestamp: i64,
    /// End timestamp in seconds.
    pub end_timestamp: i64,
    /// Initial capital amount.
    pub initial_capital: Decimal,
    /// Volatility used for optimization.
    pub volatility: Decimal,
    /// Recommended lower price bound.
    pub recommended_lower: Decimal,
    /// Recommended upper price bound.
    pub recommended_upper: Decimal,
    /// Expected PnL.
    pub expected_pnl: Decimal,
    /// Expected fees.
    pub expected_fees: Decimal,
    /// Expected IL.
    pub expected_il: Decimal,
    /// Sharpe ratio if calculated.
    pub sharpe_ratio: Option<Decimal>,
    /// Number of simulations run.
    pub simulations_run: i32,
    /// Record creation timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl OptimizationRecord {
    /// Creates an OptimizationRecord from a database row.
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            pool_id: row.try_get("pool_id")?,
            objective_type: row.try_get("objective_type")?,
            start_timestamp: row.try_get("start_timestamp")?,
            end_timestamp: row.try_get("end_timestamp")?,
            initial_capital: row.try_get("initial_capital")?,
            volatility: row.try_get("volatility")?,
            recommended_lower: row.try_get("recommended_lower")?,
            recommended_upper: row.try_get("recommended_upper")?,
            expected_pnl: row.try_get("expected_pnl")?,
            expected_fees: row.try_get("expected_fees")?,
            expected_il: row.try_get("expected_il")?,
            sharpe_ratio: row.try_get("sharpe_ratio")?,
            simulations_run: row.try_get("simulations_run")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

/// Repository for simulation CRUD operations.
#[derive(Clone)]
pub struct SimulationRepository {
    pool: Arc<PgPool>,
}

impl SimulationRepository {
    /// Creates a new SimulationRepository.
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Saves a simulation configuration.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    #[allow(clippy::too_many_arguments)]
    pub async fn save_simulation(
        &self,
        id: Uuid,
        pool_id: Option<Uuid>,
        strategy_type: &str,
        strategy_config: serde_json::Value,
        start_timestamp: i64,
        end_timestamp: i64,
        initial_capital: Decimal,
        entry_price: Decimal,
        lower_price: Decimal,
        upper_price: Decimal,
        fee_rate: Decimal,
        tx_cost: Decimal,
    ) -> Result<SimulationRecord, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO simulations (id, pool_id, strategy_type, strategy_config, 
                                    start_timestamp, end_timestamp, initial_capital,
                                    entry_price, lower_price, upper_price, fee_rate, tx_cost)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(pool_id)
        .bind(strategy_type)
        .bind(&strategy_config)
        .bind(start_timestamp)
        .bind(end_timestamp)
        .bind(initial_capital)
        .bind(entry_price)
        .bind(lower_price)
        .bind(upper_price)
        .bind(fee_rate)
        .bind(tx_cost)
        .fetch_one(self.pool.as_ref())
        .await?;
        SimulationRecord::from_row(&row)
    }

    /// Saves simulation results.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    #[allow(clippy::too_many_arguments)]
    pub async fn save_result(
        &self,
        id: Uuid,
        simulation_id: Uuid,
        final_value: Decimal,
        final_pnl: Decimal,
        total_fees: Decimal,
        total_il: Decimal,
        final_il_pct: Decimal,
        time_in_range_pct: Decimal,
        max_drawdown: Decimal,
        rebalance_count: i32,
        total_rebalance_cost: Decimal,
        hodl_value: Decimal,
        vs_hodl: Decimal,
        sharpe_ratio: Option<Decimal>,
        final_price: Decimal,
    ) -> Result<SimulationResultRecord, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO simulation_results (id, simulation_id, final_value, final_pnl,
                                           total_fees, total_il, final_il_pct, time_in_range_pct,
                                           max_drawdown, rebalance_count, total_rebalance_cost,
                                           hodl_value, vs_hodl, sharpe_ratio, final_price)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(simulation_id)
        .bind(final_value)
        .bind(final_pnl)
        .bind(total_fees)
        .bind(total_il)
        .bind(final_il_pct)
        .bind(time_in_range_pct)
        .bind(max_drawdown)
        .bind(rebalance_count)
        .bind(total_rebalance_cost)
        .bind(hodl_value)
        .bind(vs_hodl)
        .bind(sharpe_ratio)
        .bind(final_price)
        .fetch_one(self.pool.as_ref())
        .await?;
        SimulationResultRecord::from_row(&row)
    }

    /// Finds a simulation by ID.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn find_simulation_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<SimulationRecord>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM simulations WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool.as_ref())
            .await?;
        row.as_ref().map(SimulationRecord::from_row).transpose()
    }

    /// Finds simulation results by simulation ID.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn find_result_by_simulation(
        &self,
        simulation_id: Uuid,
    ) -> Result<Option<SimulationResultRecord>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM simulation_results WHERE simulation_id = $1")
            .bind(simulation_id)
            .fetch_optional(self.pool.as_ref())
            .await?;
        row.as_ref()
            .map(SimulationResultRecord::from_row)
            .transpose()
    }

    /// Finds recent simulations.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn find_recent(&self, limit: i64) -> Result<Vec<SimulationRecord>, sqlx::Error> {
        let rows = sqlx::query("SELECT * FROM simulations ORDER BY created_at DESC LIMIT $1")
            .bind(limit)
            .fetch_all(self.pool.as_ref())
            .await?;
        rows.iter().map(SimulationRecord::from_row).collect()
    }

    /// Saves an optimization result.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    #[allow(clippy::too_many_arguments)]
    pub async fn save_optimization(
        &self,
        id: Uuid,
        pool_id: Option<Uuid>,
        objective_type: &str,
        start_timestamp: i64,
        end_timestamp: i64,
        initial_capital: Decimal,
        volatility: Decimal,
        recommended_lower: Decimal,
        recommended_upper: Decimal,
        expected_pnl: Decimal,
        expected_fees: Decimal,
        expected_il: Decimal,
        sharpe_ratio: Option<Decimal>,
        simulations_run: i32,
    ) -> Result<OptimizationRecord, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO optimization_results (id, pool_id, objective_type, start_timestamp,
                                             end_timestamp, initial_capital, volatility,
                                             recommended_lower, recommended_upper,
                                             expected_pnl, expected_fees, expected_il,
                                             sharpe_ratio, simulations_run)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(pool_id)
        .bind(objective_type)
        .bind(start_timestamp)
        .bind(end_timestamp)
        .bind(initial_capital)
        .bind(volatility)
        .bind(recommended_lower)
        .bind(recommended_upper)
        .bind(expected_pnl)
        .bind(expected_fees)
        .bind(expected_il)
        .bind(sharpe_ratio)
        .bind(simulations_run)
        .fetch_one(self.pool.as_ref())
        .await?;
        OptimizationRecord::from_row(&row)
    }

    /// Finds recent optimization results.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn find_recent_optimizations(
        &self,
        limit: i64,
    ) -> Result<Vec<OptimizationRecord>, sqlx::Error> {
        let rows =
            sqlx::query("SELECT * FROM optimization_results ORDER BY created_at DESC LIMIT $1")
                .bind(limit)
                .fetch_all(self.pool.as_ref())
                .await?;
        rows.iter().map(OptimizationRecord::from_row).collect()
    }

    /// Deletes a simulation and its results.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn delete_simulation(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM simulations WHERE id = $1")
            .bind(id)
            .execute(self.pool.as_ref())
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
