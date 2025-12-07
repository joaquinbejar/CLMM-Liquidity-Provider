//! Price history repository for caching market data.

use rust_decimal::Decimal;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

/// Database record for price history.
#[derive(Debug, Clone)]
pub struct PriceRecord {
    /// Unique identifier.
    pub id: Uuid,
    /// Associated pool ID.
    pub pool_id: Option<Uuid>,
    /// Timestamp in seconds.
    pub timestamp: i64,
    /// Open price.
    pub open_price: Decimal,
    /// High price.
    pub high_price: Decimal,
    /// Low price.
    pub low_price: Decimal,
    /// Close price.
    pub close_price: Decimal,
    /// Trading volume.
    pub volume: Option<Decimal>,
    /// Pool liquidity.
    pub liquidity: Option<Decimal>,
    /// Record creation timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl PriceRecord {
    /// Creates a PriceRecord from a database row.
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            pool_id: row.try_get("pool_id")?,
            timestamp: row.try_get("timestamp")?,
            open_price: row.try_get("open_price")?,
            high_price: row.try_get("high_price")?,
            low_price: row.try_get("low_price")?,
            close_price: row.try_get("close_price")?,
            volume: row.try_get("volume")?,
            liquidity: row.try_get("liquidity")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

/// Repository for price history CRUD operations.
#[derive(Clone)]
pub struct PriceRepository {
    pool: Arc<PgPool>,
}

impl PriceRepository {
    /// Creates a new PriceRepository.
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Saves a price record.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    #[allow(clippy::too_many_arguments)]
    pub async fn save(
        &self,
        id: Uuid,
        pool_id: Option<Uuid>,
        timestamp: i64,
        open_price: Decimal,
        high_price: Decimal,
        low_price: Decimal,
        close_price: Decimal,
        volume: Option<Decimal>,
        liquidity: Option<Decimal>,
    ) -> Result<PriceRecord, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO price_history (id, pool_id, timestamp, open_price, high_price,
                                       low_price, close_price, volume, liquidity)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (pool_id, timestamp) DO UPDATE SET
                open_price = EXCLUDED.open_price,
                high_price = EXCLUDED.high_price,
                low_price = EXCLUDED.low_price,
                close_price = EXCLUDED.close_price,
                volume = EXCLUDED.volume,
                liquidity = EXCLUDED.liquidity
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(pool_id)
        .bind(timestamp)
        .bind(open_price)
        .bind(high_price)
        .bind(low_price)
        .bind(close_price)
        .bind(volume)
        .bind(liquidity)
        .fetch_one(self.pool.as_ref())
        .await?;
        PriceRecord::from_row(&row)
    }

    /// Finds price history for a pool within a time range.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn find_by_pool_and_range(
        &self,
        pool_id: Uuid,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Result<Vec<PriceRecord>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM price_history 
            WHERE pool_id = $1 AND timestamp >= $2 AND timestamp <= $3
            ORDER BY timestamp ASC
            "#,
        )
        .bind(pool_id)
        .bind(start_timestamp)
        .bind(end_timestamp)
        .fetch_all(self.pool.as_ref())
        .await?;
        rows.iter().map(PriceRecord::from_row).collect()
    }

    /// Finds the latest price for a pool.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn find_latest(&self, pool_id: Uuid) -> Result<Option<PriceRecord>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT * FROM price_history 
            WHERE pool_id = $1
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(pool_id)
        .fetch_optional(self.pool.as_ref())
        .await?;
        row.as_ref().map(PriceRecord::from_row).transpose()
    }

    /// Checks if price data exists for a pool in a time range.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn has_data_for_range(
        &self,
        pool_id: Uuid,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Result<bool, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM price_history 
            WHERE pool_id = $1 AND timestamp >= $2 AND timestamp <= $3
            "#,
        )
        .bind(pool_id)
        .bind(start_timestamp)
        .bind(end_timestamp)
        .fetch_one(self.pool.as_ref())
        .await?;
        Ok(count.0 > 0)
    }

    /// Deletes price history for a pool.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn delete_by_pool(&self, pool_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM price_history WHERE pool_id = $1")
            .bind(pool_id)
            .execute(self.pool.as_ref())
            .await?;
        Ok(result.rows_affected())
    }

    /// Deletes old price history before a timestamp.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn delete_before(&self, before_timestamp: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM price_history WHERE timestamp < $1")
            .bind(before_timestamp)
            .execute(self.pool.as_ref())
            .await?;
        Ok(result.rows_affected())
    }
}
