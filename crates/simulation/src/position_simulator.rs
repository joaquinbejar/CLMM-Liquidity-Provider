//! High-level position simulator.
//!
//! This module provides a simplified interface for simulating LP positions
//! over a price path, returning comprehensive results.

use crate::event::{EventLog, SimulationEvent};
use crate::liquidity::LiquidityModel;
use crate::price_path::PricePathGenerator;
use crate::state::{SimulationConfig, SimulationSummary};
use crate::volume::VolumeModel;
use clmm_lp_domain::metrics::impermanent_loss::calculate_il_concentrated;
use clmm_lp_domain::value_objects::price::Price;
use clmm_lp_domain::value_objects::price_range::PriceRange;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive as _;

/// Result of a position simulation.
#[derive(Debug, Clone)]
pub struct PositionSimulationResult {
    /// Summary of the simulation.
    pub summary: SimulationSummary,
    /// Event log from the simulation.
    pub events: Vec<SimulationEvent>,
    /// Price path used.
    pub prices: Vec<Price>,
    /// Step-by-step PnL values.
    pub pnl_history: Vec<Decimal>,
    /// Step-by-step IL values.
    pub il_history: Vec<Decimal>,
    /// Step-by-step fee values.
    pub fee_history: Vec<Decimal>,
}

/// Simulates a static LP position (no rebalancing).
///
/// # Arguments
/// * `config` - Simulation configuration
/// * `price_path` - Price path generator
/// * `volume_model` - Volume model for fee calculation
/// * `liquidity_model` - Liquidity model
///
/// # Returns
/// Simulation result with full metrics
pub fn simulate_position<P, V, L>(
    config: &SimulationConfig,
    price_path: &mut P,
    volume_model: &mut V,
    liquidity_model: &L,
) -> PositionSimulationResult
where
    P: PricePathGenerator,
    V: VolumeModel,
    L: LiquidityModel,
{
    let prices = price_path.generate(config.steps);

    if prices.is_empty() {
        return empty_result(config);
    }

    let entry_price = prices[0];
    let range = &config.initial_range;

    let mut event_log = EventLog::new();
    let mut cumulative_fees = Decimal::ZERO;
    let mut steps_in_range: u64 = 0;
    let mut max_il = Decimal::ZERO;
    let mut max_value = config.initial_capital;
    let mut max_drawdown = Decimal::ZERO;

    let mut pnl_history = Vec::with_capacity(prices.len());
    let mut il_history = Vec::with_capacity(prices.len());
    let mut fee_history = Vec::with_capacity(prices.len());

    let mut was_in_range = is_in_range(&entry_price, range);

    // Record position opened
    event_log.record(SimulationEvent::position_opened(
        0,
        entry_price,
        config.initial_capital,
        range.clone(),
    ));

    for (step, price) in prices.iter().enumerate() {
        let in_range = is_in_range(price, range);

        // Track range transitions
        if in_range && !was_in_range {
            event_log.record(SimulationEvent::back_in_range(
                step as u64,
                *price,
                range.clone(),
            ));
        } else if !in_range && was_in_range {
            event_log.record(SimulationEvent::out_of_range(
                step as u64,
                *price,
                range.clone(),
            ));
        }
        was_in_range = in_range;

        if in_range {
            steps_in_range += 1;

            // Calculate fees for this step
            let volume = volume_model.get_volume(step);
            let pool_liquidity = liquidity_model.get_liquidity(step);

            let step_fees = if pool_liquidity > 0 {
                let lp_share = Decimal::from(config.pool_liquidity) / Decimal::from(pool_liquidity);
                volume * config.fee_rate * lp_share
            } else {
                Decimal::ZERO
            };

            cumulative_fees += step_fees;

            if step_fees > Decimal::ZERO {
                event_log.record(SimulationEvent::fee_collection(
                    step as u64,
                    *price,
                    step_fees,
                    cumulative_fees,
                ));
            }
        }

        // Calculate IL
        let il_decimal = calculate_il_concentrated(
            entry_price.value,
            price.value,
            range.lower_price.value,
            range.upper_price.value,
        )
        .unwrap_or(Decimal::ZERO);

        if il_decimal < max_il {
            max_il = il_decimal;
        }

        // Calculate position value
        let il_amount = config.initial_capital * il_decimal.abs();
        let position_value = config.initial_capital - il_amount + cumulative_fees;
        let net_pnl = position_value - config.initial_capital;

        // Track max value and drawdown
        if position_value > max_value {
            max_value = position_value;
        }
        let drawdown = if max_value.is_zero() {
            Decimal::ZERO
        } else {
            (position_value - max_value) / max_value
        };
        if drawdown < max_drawdown {
            max_drawdown = drawdown;
        }

        pnl_history.push(net_pnl);
        il_history.push(il_decimal);
        fee_history.push(cumulative_fees);
    }

    let final_price = *prices.last().unwrap_or(&entry_price);

    let final_il_decimal = calculate_il_concentrated(
        entry_price.value,
        final_price.value,
        range.lower_price.value,
        range.upper_price.value,
    )
    .unwrap_or(Decimal::ZERO);

    let final_price_ratio = if entry_price.value.is_zero() {
        1.0
    } else {
        (final_price.value / entry_price.value)
            .to_f64()
            .unwrap_or(1.0)
    };

    let il_amount = config.initial_capital * final_il_decimal.abs();
    let final_value = config.initial_capital - il_amount + cumulative_fees;
    let net_pnl = final_value - config.initial_capital;
    let net_pnl_pct = if config.initial_capital.is_zero() {
        Decimal::ZERO
    } else {
        net_pnl / config.initial_capital
    };

    // HODL comparison
    let hodl_value =
        config.initial_capital * Decimal::try_from(final_price_ratio).unwrap_or(Decimal::ONE);
    let vs_hodl = final_value - hodl_value;

    // Record position closed
    event_log.record(SimulationEvent::position_closed(
        prices.len() as u64,
        final_price,
        final_value,
        cumulative_fees,
        final_il_decimal,
        net_pnl,
    ));

    let summary = SimulationSummary {
        config: config.clone(),
        entry_price,
        final_price,
        total_steps: prices.len() as u64,
        steps_in_range,
        final_value,
        total_fees: cumulative_fees,
        final_il_pct: final_il_decimal,
        net_pnl,
        net_pnl_pct,
        rebalance_count: 0,
        total_rebalance_cost: Decimal::ZERO,
        max_il_pct: max_il,
        max_drawdown_pct: max_drawdown,
        hodl_value,
        vs_hodl,
    };

    PositionSimulationResult {
        summary,
        events: event_log.events().to_vec(),
        prices,
        pnl_history,
        il_history,
        fee_history,
    }
}

/// Checks if a price is within a range.
fn is_in_range(price: &Price, range: &PriceRange) -> bool {
    price.value >= range.lower_price.value && price.value <= range.upper_price.value
}

/// Creates an empty result for edge cases.
fn empty_result(config: &SimulationConfig) -> PositionSimulationResult {
    let entry_price = Price::new(Decimal::ZERO);
    let summary = SimulationSummary {
        config: config.clone(),
        entry_price,
        final_price: entry_price,
        total_steps: 0,
        steps_in_range: 0,
        final_value: config.initial_capital,
        total_fees: Decimal::ZERO,
        final_il_pct: Decimal::ZERO,
        net_pnl: Decimal::ZERO,
        net_pnl_pct: Decimal::ZERO,
        rebalance_count: 0,
        total_rebalance_cost: Decimal::ZERO,
        max_il_pct: Decimal::ZERO,
        max_drawdown_pct: Decimal::ZERO,
        hodl_value: config.initial_capital,
        vs_hodl: Decimal::ZERO,
    };

    PositionSimulationResult {
        summary,
        events: Vec::new(),
        prices: Vec::new(),
        pnl_history: Vec::new(),
        il_history: Vec::new(),
        fee_history: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::liquidity::ConstantLiquidity;
    use crate::price_path::DeterministicPricePath;
    use crate::volume::ConstantVolume;
    use rust_decimal_macros::dec;

    #[test]
    fn test_simulate_position_flat_price() {
        let range = PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110)));
        let config = SimulationConfig::new(dec!(1000), range)
            .with_steps(10)
            .with_fee_rate(dec!(0.003))
            .with_pool_liquidity(1_000_000);

        let prices = vec![dec!(100); 10];
        let mut price_path = DeterministicPricePath::new(prices);
        let mut volume_model = ConstantVolume::new(dec!(10000));
        let liquidity_model = ConstantLiquidity::new(1_000_000);

        let result = simulate_position(
            &config,
            &mut price_path,
            &mut volume_model,
            &liquidity_model,
        );

        assert_eq!(result.summary.total_steps, 10);
        assert_eq!(result.summary.steps_in_range, 10);
        assert!(result.summary.total_fees > Decimal::ZERO);
        assert_eq!(result.summary.final_il_pct, Decimal::ZERO);
        assert_eq!(result.summary.rebalance_count, 0);
    }

    #[test]
    fn test_simulate_position_price_movement() {
        let range = PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110)));
        let config = SimulationConfig::new(dec!(1000), range)
            .with_steps(5)
            .with_fee_rate(dec!(0.003));

        let prices = vec![dec!(100), dec!(105), dec!(110), dec!(115), dec!(120)];
        let mut price_path = DeterministicPricePath::new(prices);
        let mut volume_model = ConstantVolume::new(dec!(10000));
        let liquidity_model = ConstantLiquidity::new(1_000_000);

        let result = simulate_position(
            &config,
            &mut price_path,
            &mut volume_model,
            &liquidity_model,
        );

        // Price goes out of range at step 3 (115) and 4 (120)
        assert!(result.summary.steps_in_range < result.summary.total_steps);
        // IL should be negative
        assert!(result.summary.final_il_pct < Decimal::ZERO);
    }

    #[test]
    fn test_simulate_position_events() {
        let range = PriceRange::new(Price::new(dec!(95)), Price::new(dec!(105)));
        let config = SimulationConfig::new(dec!(1000), range)
            .with_steps(5)
            .with_fee_rate(dec!(0.003));

        let prices = vec![dec!(100), dec!(102), dec!(108), dec!(103), dec!(100)];
        let mut price_path = DeterministicPricePath::new(prices);
        let mut volume_model = ConstantVolume::new(dec!(10000));
        let liquidity_model = ConstantLiquidity::new(1_000_000);

        let result = simulate_position(
            &config,
            &mut price_path,
            &mut volume_model,
            &liquidity_model,
        );

        // Should have: PositionOpened, some FeeCollections, OutOfRange, BackInRange, PositionClosed
        assert!(!result.events.is_empty());

        // First event should be PositionOpened
        assert!(matches!(
            result.events.first().unwrap().event_type,
            crate::event::SimulationEventType::PositionOpened
        ));

        // Last event should be PositionClosed
        assert!(matches!(
            result.events.last().unwrap().event_type,
            crate::event::SimulationEventType::PositionClosed
        ));
    }
}
