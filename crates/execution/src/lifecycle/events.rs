//! Lifecycle events for position tracking.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;

/// Type of lifecycle event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleEventType {
    /// Position was opened.
    PositionOpened,
    /// Liquidity was increased.
    LiquidityIncreased,
    /// Liquidity was decreased.
    LiquidityDecreased,
    /// Position was rebalanced.
    Rebalanced,
    /// Fees were collected.
    FeesCollected,
    /// Position was closed.
    PositionClosed,
}

/// A lifecycle event for a position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    /// Event ID.
    pub id: String,
    /// Event type.
    pub event_type: LifecycleEventType,
    /// Position address.
    pub position: Pubkey,
    /// Pool address.
    pub pool: Pubkey,
    /// Transaction signature.
    pub signature: Option<Signature>,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event-specific data.
    pub data: EventData,
}

impl LifecycleEvent {
    /// Creates a new lifecycle event.
    pub fn new(
        event_type: LifecycleEventType,
        position: Pubkey,
        pool: Pubkey,
        data: EventData,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            position,
            pool,
            signature: None,
            timestamp: chrono::Utc::now(),
            data,
        }
    }

    /// Sets the transaction signature.
    #[must_use]
    pub fn with_signature(mut self, signature: Signature) -> Self {
        self.signature = Some(signature);
        self
    }
}

/// Event-specific data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventData {
    /// Position opened data.
    PositionOpened(PositionOpenedData),
    /// Liquidity change data.
    LiquidityChange(LiquidityChangeData),
    /// Rebalance data.
    Rebalance(RebalanceData),
    /// Fees collected data.
    FeesCollected(FeesCollectedData),
    /// Position closed data.
    PositionClosed(PositionClosedData),
}

/// Data for position opened event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionOpenedData {
    /// Lower tick.
    pub tick_lower: i32,
    /// Upper tick.
    pub tick_upper: i32,
    /// Initial liquidity.
    pub liquidity: u128,
    /// Token A amount deposited.
    pub amount_a: u64,
    /// Token B amount deposited.
    pub amount_b: u64,
    /// Entry price.
    pub entry_price: Decimal,
    /// Entry value in USD.
    pub entry_value_usd: Decimal,
}

/// Data for liquidity change event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityChangeData {
    /// Whether liquidity was increased (true) or decreased (false).
    pub is_increase: bool,
    /// Liquidity delta.
    pub liquidity_delta: u128,
    /// Token A amount.
    pub amount_a: u64,
    /// Token B amount.
    pub amount_b: u64,
    /// New total liquidity.
    pub new_liquidity: u128,
}

/// Data for rebalance event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceData {
    /// Old lower tick.
    pub old_tick_lower: i32,
    /// Old upper tick.
    pub old_tick_upper: i32,
    /// New lower tick.
    pub new_tick_lower: i32,
    /// New upper tick.
    pub new_tick_upper: i32,
    /// Liquidity before rebalance.
    pub old_liquidity: u128,
    /// Liquidity after rebalance.
    pub new_liquidity: u128,
    /// Transaction cost in lamports.
    pub tx_cost_lamports: u64,
    /// IL at time of rebalance.
    pub il_at_rebalance: Decimal,
    /// Reason for rebalance.
    pub reason: RebalanceReason,
}

/// Reason for rebalancing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RebalanceReason {
    /// Price exited range.
    RangeExit,
    /// IL exceeded threshold.
    ILThreshold,
    /// Periodic rebalance.
    Periodic,
    /// Manual trigger.
    Manual,
    /// Optimization suggested.
    Optimization,
}

/// Data for fees collected event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeesCollectedData {
    /// Token A fees collected.
    pub fees_a: u64,
    /// Token B fees collected.
    pub fees_b: u64,
    /// Fees value in USD.
    pub fees_usd: Decimal,
}

/// Data for position closed event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionClosedData {
    /// Final liquidity removed.
    pub liquidity_removed: u128,
    /// Token A received.
    pub amount_a: u64,
    /// Token B received.
    pub amount_b: u64,
    /// Total fees earned over lifetime.
    pub total_fees_a: u64,
    /// Total fees earned over lifetime.
    pub total_fees_b: u64,
    /// Final PnL in USD.
    pub final_pnl_usd: Decimal,
    /// Final PnL percentage.
    pub final_pnl_pct: Decimal,
    /// Total IL over lifetime.
    pub total_il_pct: Decimal,
    /// Position duration in hours.
    pub duration_hours: u64,
    /// Reason for closing.
    pub reason: CloseReason,
}

/// Reason for closing a position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloseReason {
    /// Manual close.
    Manual,
    /// IL exceeded threshold.
    ILThreshold,
    /// PnL target reached.
    PnLTarget,
    /// Emergency exit.
    Emergency,
    /// Strategy ended.
    StrategyEnded,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_event_creation() {
        let event = LifecycleEvent::new(
            LifecycleEventType::PositionOpened,
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            EventData::PositionOpened(PositionOpenedData {
                tick_lower: -1000,
                tick_upper: 1000,
                liquidity: 1000000,
                amount_a: 1000000000,
                amount_b: 100000000,
                entry_price: Decimal::new(100, 0),
                entry_value_usd: Decimal::new(1000, 0),
            }),
        );

        assert_eq!(event.event_type, LifecycleEventType::PositionOpened);
        assert!(event.signature.is_none());
    }
}
