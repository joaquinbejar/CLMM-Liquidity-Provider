//! Pool repository for CLMM pool persistence.

use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

/// Database record for a CLMM pool.
#[derive(Debug, Clone)]
pub struct PoolRecord {
    /// Unique identifier.
    pub id: Uuid,
    /// Protocol name (raydium, orca, meteora).
    pub protocol: String,
    /// On-chain pool address.
    pub address: String,
    /// Token A mint address.
    pub token_mint_a: String,
    /// Token B mint address.
    pub token_mint_b: String,
    /// Token A symbol.
    pub symbol_a: String,
    /// Token B symbol.
    pub symbol_b: String,
    /// Token A decimals.
    pub decimals_a: i16,
    /// Token B decimals.
    pub decimals_b: i16,
    /// Fee tier in basis points.
    pub fee_tier: i32,
    /// Tick spacing for the pool.
    pub tick_spacing: i32,
    /// Record creation timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Record update timestamp.
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl PoolRecord {
    /// Creates a PoolRecord from a database row.
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            protocol: row.try_get("protocol")?,
            address: row.try_get("address")?,
            token_mint_a: row.try_get("token_mint_a")?,
            token_mint_b: row.try_get("token_mint_b")?,
            symbol_a: row.try_get("symbol_a")?,
            symbol_b: row.try_get("symbol_b")?,
            decimals_a: row.try_get("decimals_a")?,
            decimals_b: row.try_get("decimals_b")?,
            fee_tier: row.try_get("fee_tier")?,
            tick_spacing: row.try_get("tick_spacing")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

/// Repository for pool CRUD operations.
#[derive(Clone)]
pub struct PoolRepository {
    pool: Arc<PgPool>,
}

impl PoolRepository {
    /// Creates a new PoolRepository.
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Finds a pool by its ID.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<PoolRecord>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM pools WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool.as_ref())
            .await?;
        row.as_ref().map(PoolRecord::from_row).transpose()
    }

    /// Finds a pool by its on-chain address.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn find_by_address(&self, address: &str) -> Result<Option<PoolRecord>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM pools WHERE address = $1")
            .bind(address)
            .fetch_optional(self.pool.as_ref())
            .await?;
        row.as_ref().map(PoolRecord::from_row).transpose()
    }

    /// Finds all pools for a given protocol.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn find_by_protocol(&self, protocol: &str) -> Result<Vec<PoolRecord>, sqlx::Error> {
        let rows = sqlx::query("SELECT * FROM pools WHERE protocol = $1 ORDER BY created_at DESC")
            .bind(protocol)
            .fetch_all(self.pool.as_ref())
            .await?;
        rows.iter().map(PoolRecord::from_row).collect()
    }

    /// Finds all pools.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn find_all(&self) -> Result<Vec<PoolRecord>, sqlx::Error> {
        let rows = sqlx::query("SELECT * FROM pools ORDER BY created_at DESC")
            .fetch_all(self.pool.as_ref())
            .await?;
        rows.iter().map(PoolRecord::from_row).collect()
    }

    /// Creates or updates a pool record.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert(
        &self,
        id: Uuid,
        protocol: &str,
        address: &str,
        token_mint_a: &str,
        token_mint_b: &str,
        symbol_a: &str,
        symbol_b: &str,
        decimals_a: i16,
        decimals_b: i16,
        fee_tier: i32,
        tick_spacing: i32,
    ) -> Result<PoolRecord, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO pools (id, protocol, address, token_mint_a, token_mint_b, 
                              symbol_a, symbol_b, decimals_a, decimals_b, fee_tier, tick_spacing)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (address) DO UPDATE SET
                protocol = EXCLUDED.protocol,
                token_mint_a = EXCLUDED.token_mint_a,
                token_mint_b = EXCLUDED.token_mint_b,
                symbol_a = EXCLUDED.symbol_a,
                symbol_b = EXCLUDED.symbol_b,
                decimals_a = EXCLUDED.decimals_a,
                decimals_b = EXCLUDED.decimals_b,
                fee_tier = EXCLUDED.fee_tier,
                tick_spacing = EXCLUDED.tick_spacing,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(protocol)
        .bind(address)
        .bind(token_mint_a)
        .bind(token_mint_b)
        .bind(symbol_a)
        .bind(symbol_b)
        .bind(decimals_a)
        .bind(decimals_b)
        .bind(fee_tier)
        .bind(tick_spacing)
        .fetch_one(self.pool.as_ref())
        .await?;
        PoolRecord::from_row(&row)
    }

    /// Deletes a pool by ID.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM pools WHERE id = $1")
            .bind(id)
            .execute(self.pool.as_ref())
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
