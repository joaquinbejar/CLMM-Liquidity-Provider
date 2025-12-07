//! Simulation events for tracking what happens during a backtest.
//!
//! This module defines event types that can occur during a simulation,
//! such as swaps, rebalances, fee collections, and position changes.

use clmm_lp_domain::value_objects::price::Price;
use clmm_lp_domain::value_objects::price_range::PriceRange;
use rust_decimal::Decimal;

/// Types of events that can occur during simulation.
#[derive(Debug, Clone, PartialEq)]
pub enum SimulationEventType {
    /// Position was opened.
    PositionOpened,
    /// Position was closed.
    PositionClosed,
    /// Position was rebalanced to a new range.
    Rebalance,
    /// Fees were collected.
    FeeCollection,
    /// Price moved out of range.
    OutOfRange,
    /// Price moved back into range.
    BackInRange,
    /// Swap occurred in the pool.
    Swap,
    /// Liquidity was added to the pool.
    LiquidityAdded,
    /// Liquidity was removed from the pool.
    LiquidityRemoved,
}

/// A simulation event with full context.
#[derive(Debug, Clone)]
pub struct SimulationEvent {
    /// Step number when event occurred.
    pub step: u64,
    /// Timestamp in seconds (if available).
    pub timestamp: Option<u64>,
    /// Type of event.
    pub event_type: SimulationEventType,
    /// Price at the time of event.
    pub price: Price,
    /// Additional event-specific data.
    pub data: EventData,
}

/// Event-specific data payload.
#[derive(Debug, Clone)]
pub enum EventData {
    /// No additional data.
    None,
    /// Position opened data.
    PositionOpened {
        /// Initial capital invested.
        capital: Decimal,
        /// Initial price range.
        range: PriceRange,
    },
    /// Position closed data.
    PositionClosed {
        /// Final position value.
        final_value: Decimal,
        /// Total fees earned.
        total_fees: Decimal,
        /// Final IL percentage.
        final_il_pct: Decimal,
        /// Net PnL.
        net_pnl: Decimal,
    },
    /// Rebalance event data.
    Rebalance {
        /// Previous price range.
        old_range: PriceRange,
        /// New price range.
        new_range: PriceRange,
        /// Reason for rebalance.
        reason: String,
        /// Transaction cost.
        cost: Decimal,
    },
    /// Fee collection data.
    FeeCollection {
        /// Amount of fees collected.
        amount: Decimal,
        /// Cumulative fees after collection.
        cumulative: Decimal,
    },
    /// Range transition data.
    RangeTransition {
        /// Whether entering (true) or exiting (false) range.
        entering: bool,
        /// Current range.
        range: PriceRange,
    },
    /// Swap event data.
    Swap {
        /// Volume of the swap.
        volume: Decimal,
        /// Direction: true = buy token A, false = sell token A.
        is_buy: bool,
        /// Price impact of the swap.
        price_impact: Decimal,
    },
}

impl SimulationEvent {
    /// Creates a new position opened event.
    #[must_use]
    pub fn position_opened(step: u64, price: Price, capital: Decimal, range: PriceRange) -> Self {
        Self {
            step,
            timestamp: None,
            event_type: SimulationEventType::PositionOpened,
            price,
            data: EventData::PositionOpened { capital, range },
        }
    }

    /// Creates a new position closed event.
    #[must_use]
    pub fn position_closed(
        step: u64,
        price: Price,
        final_value: Decimal,
        total_fees: Decimal,
        final_il_pct: Decimal,
        net_pnl: Decimal,
    ) -> Self {
        Self {
            step,
            timestamp: None,
            event_type: SimulationEventType::PositionClosed,
            price,
            data: EventData::PositionClosed {
                final_value,
                total_fees,
                final_il_pct,
                net_pnl,
            },
        }
    }

    /// Creates a new rebalance event.
    #[must_use]
    pub fn rebalance(
        step: u64,
        price: Price,
        old_range: PriceRange,
        new_range: PriceRange,
        reason: String,
        cost: Decimal,
    ) -> Self {
        Self {
            step,
            timestamp: None,
            event_type: SimulationEventType::Rebalance,
            price,
            data: EventData::Rebalance {
                old_range,
                new_range,
                reason,
                cost,
            },
        }
    }

    /// Creates a new fee collection event.
    #[must_use]
    pub fn fee_collection(step: u64, price: Price, amount: Decimal, cumulative: Decimal) -> Self {
        Self {
            step,
            timestamp: None,
            event_type: SimulationEventType::FeeCollection,
            price,
            data: EventData::FeeCollection { amount, cumulative },
        }
    }

    /// Creates an out-of-range event.
    #[must_use]
    pub fn out_of_range(step: u64, price: Price, range: PriceRange) -> Self {
        Self {
            step,
            timestamp: None,
            event_type: SimulationEventType::OutOfRange,
            price,
            data: EventData::RangeTransition {
                entering: false,
                range,
            },
        }
    }

    /// Creates a back-in-range event.
    #[must_use]
    pub fn back_in_range(step: u64, price: Price, range: PriceRange) -> Self {
        Self {
            step,
            timestamp: None,
            event_type: SimulationEventType::BackInRange,
            price,
            data: EventData::RangeTransition {
                entering: true,
                range,
            },
        }
    }

    /// Sets the timestamp for this event.
    #[must_use]
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
}

/// Event log for collecting all events during simulation.
#[derive(Debug, Default)]
pub struct EventLog {
    /// All recorded events.
    events: Vec<SimulationEvent>,
}

impl EventLog {
    /// Creates a new empty event log.
    #[must_use]
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Records an event.
    pub fn record(&mut self, event: SimulationEvent) {
        self.events.push(event);
    }

    /// Returns all events.
    #[must_use]
    pub fn events(&self) -> &[SimulationEvent] {
        &self.events
    }

    /// Returns events of a specific type.
    #[must_use]
    pub fn events_of_type(&self, event_type: SimulationEventType) -> Vec<&SimulationEvent> {
        self.events
            .iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    /// Returns the count of events by type.
    #[must_use]
    pub fn count_by_type(&self, event_type: SimulationEventType) -> usize {
        self.events
            .iter()
            .filter(|e| e.event_type == event_type)
            .count()
    }

    /// Returns total rebalance count.
    #[must_use]
    pub fn rebalance_count(&self) -> usize {
        self.count_by_type(SimulationEventType::Rebalance)
    }

    /// Returns total fee collection events.
    #[must_use]
    pub fn fee_collection_count(&self) -> usize {
        self.count_by_type(SimulationEventType::FeeCollection)
    }

    /// Clears all events.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_event_log_record_and_query() {
        let mut log = EventLog::new();

        let price = Price::new(dec!(100));
        let range = PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110)));

        log.record(SimulationEvent::position_opened(
            0,
            price,
            dec!(1000),
            range.clone(),
        ));
        log.record(SimulationEvent::fee_collection(
            1,
            price,
            dec!(10),
            dec!(10),
        ));
        log.record(SimulationEvent::fee_collection(
            2,
            price,
            dec!(15),
            dec!(25),
        ));
        log.record(SimulationEvent::out_of_range(
            3,
            Price::new(dec!(120)),
            range.clone(),
        ));

        assert_eq!(log.events().len(), 4);
        assert_eq!(log.fee_collection_count(), 2);
        assert_eq!(log.count_by_type(SimulationEventType::OutOfRange), 1);
    }

    #[test]
    fn test_rebalance_event() {
        let price = Price::new(dec!(100));
        let old_range = PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110)));
        let new_range = PriceRange::new(Price::new(dec!(95)), Price::new(dec!(105)));

        let event = SimulationEvent::rebalance(
            5,
            price,
            old_range,
            new_range,
            "Price threshold exceeded".to_string(),
            dec!(1.5),
        );

        assert_eq!(event.event_type, SimulationEventType::Rebalance);
        assert_eq!(event.step, 5);
    }
}
