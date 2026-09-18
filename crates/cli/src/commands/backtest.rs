//! Backtest command implementation.
//!
//! Provides backtesting functionality for LP strategies
//! using historical price data.

use crate::output::{BacktestReport, print_backtest_report};
use anyhow::Result;
use clmm_lp_data::prelude::*;
use clmm_lp_domain::entities::token::Token;
use clmm_lp_domain::value_objects::price::Price;
use clmm_lp_domain::value_objects::price_range::PriceRange;
use clmm_lp_simulation::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use tracing::info;

/// Arguments for the backtest command.
#[derive(Debug, Clone)]
pub struct BacktestArgs {
    /// Token A symbol.
    pub symbol_a: String,
    /// Token A mint address.
    pub mint_a: String,
    /// Token B symbol.
    pub symbol_b: String,
    /// Token B mint address.
    pub mint_b: String,
    /// Number of days to backtest.
    pub days: u64,
    /// Lower price bound.
    pub lower_price: Decimal,
    /// Upper price bound.
    pub upper_price: Decimal,
    /// Initial capital in USD.
    pub capital: Decimal,
    /// Rebalancing strategy.
    pub strategy: StrategyType,
    /// Rebalance interval (for periodic strategy).
    pub rebalance_interval: u64,
    /// Price threshold (for threshold strategy).
    pub price_threshold: Decimal,
    /// Transaction cost per rebalance.
    pub tx_cost: Decimal,
    /// Output format.
    pub format: OutputFormat,
}

/// Strategy type for backtesting.
#[derive(Debug, Clone, Copy, Default)]
pub enum StrategyType {
    /// No rebalancing.
    #[default]
    Static,
    /// Periodic rebalancing.
    Periodic,
    /// Threshold-based rebalancing.
    Threshold,
    /// IL-limit rebalancing.
    ILLimit,
}

/// Output format.
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    /// Human-readable table.
    #[default]
    Table,
    /// JSON format.
    Json,
    /// CSV format.
    Csv,
}

impl Default for BacktestArgs {
    fn default() -> Self {
        Self {
            symbol_a: "SOL".to_string(),
            mint_a: "So11111111111111111111111111111111111111112".to_string(),
            symbol_b: "USDC".to_string(),
            mint_b: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            days: 30,
            lower_price: Decimal::from(90),
            upper_price: Decimal::from(110),
            capital: Decimal::from(1000),
            strategy: StrategyType::Static,
            rebalance_interval: 24,
            price_threshold: Decimal::from_f64(0.05).unwrap(),
            tx_cost: Decimal::from_f64(0.001).unwrap(),
            format: OutputFormat::Table,
        }
    }
}

/// Runs the backtest command.
pub async fn run_backtest(args: BacktestArgs) -> Result<()> {
    info!(
        "Running backtest for {}/{} over {} days",
        args.symbol_a, args.symbol_b, args.days
    );
    info!(
        "Range: {} - {}, Capital: {}",
        args.lower_price, args.upper_price, args.capital
    );

    // Create tokens
    let token_a = Token::new(&args.mint_a, &args.symbol_a, 9, &args.symbol_a);
    let token_b = Token::new(&args.mint_b, &args.symbol_b, 6, &args.symbol_b);

    // Try to fetch data
    let api_key = std::env::var("BIRDEYE_API_KEY").ok();

    let prices = if let Some(key) = api_key {
        let provider = BirdeyeProvider::new(key);

        let end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let start_time = end_time - (args.days * 24 * 3600);

        match provider
            .get_price_history(&token_a, &token_b, start_time, end_time, 3600)
            .await
        {
            Ok(candles) => {
                info!("Fetched {} candles", candles.len());
                candles.iter().map(|c| c.close).collect()
            }
            Err(e) => {
                info!("Failed to fetch data: {}. Using synthetic data.", e);
                generate_synthetic_prices(args.days as usize * 24)
            }
        }
    } else {
        info!("No API key found. Using synthetic data.");
        generate_synthetic_prices(args.days as usize * 24)
    };

    // Run simulation
    let report = run_simulation(&args, &prices)?;

    // Output the report
    match args.format {
        OutputFormat::Table => print_backtest_report(&report),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        OutputFormat::Csv => print_csv_backtest(&report),
    }

    Ok(())
}

/// Runs the simulation with the given prices.
fn run_simulation(args: &BacktestArgs, prices: &[Price]) -> Result<BacktestReport> {
    let range = PriceRange::new(Price::new(args.lower_price), Price::new(args.upper_price));

    let entry_price = prices
        .first()
        .map(|p| p.value)
        .unwrap_or(Decimal::from(100));

    // Create simulation config
    let config = SimulationConfig::new(args.capital, range.clone())
        .with_fee_rate(Decimal::from_f64(0.003).unwrap())
        .with_rebalance_cost(args.tx_cost)
        .with_pool_liquidity(1_000_000_000)
        .with_steps(prices.len())
        .with_step_duration(3600);

    // Create strategy based on type
    let range_width = Decimal::from_f64(0.10).unwrap();

    // Create price path generator
    let mut price_path = DeterministicPricePath::from_prices(prices.to_vec());

    // Create volume and liquidity models
    let mut volume_model = ConstantVolume::new(Decimal::from(1_000_000));
    let liquidity_model = ConstantLiquidity::new(1_000_000_000);

    // Run simulation with appropriate strategy
    let result = match args.strategy {
        StrategyType::Static => {
            let strategy = StaticRange;
            simulate_with_strategy(
                &config,
                &mut price_path,
                &mut volume_model,
                &liquidity_model,
                &strategy,
            )
        }
        StrategyType::Periodic => {
            let strategy = PeriodicRebalance::new(args.rebalance_interval, range_width);
            simulate_with_strategy(
                &config,
                &mut price_path,
                &mut volume_model,
                &liquidity_model,
                &strategy,
            )
        }
        StrategyType::Threshold => {
            let strategy = ThresholdRebalance::new(args.price_threshold, range_width);
            simulate_with_strategy(
                &config,
                &mut price_path,
                &mut volume_model,
                &liquidity_model,
                &strategy,
            )
        }
        StrategyType::ILLimit => {
            let strategy = ILLimitStrategy::new(Decimal::from_f64(0.05).unwrap(), range_width);
            simulate_with_strategy(
                &config,
                &mut price_path,
                &mut volume_model,
                &liquidity_model,
                &strategy,
            )
        }
    };

    // Calculate additional metrics
    let final_price = prices.last().map(|p| p.value).unwrap_or(entry_price);
    let hodl_return = if entry_price.is_zero() {
        Decimal::ZERO
    } else {
        (final_price - entry_price) / entry_price * Decimal::from(100)
    };

    let total_return = if args.capital.is_zero() {
        Decimal::ZERO
    } else {
        result.summary.net_pnl / args.capital * Decimal::from(100)
    };
    let vs_hodl = total_return - hodl_return;

    Ok(BacktestReport {
        pair: format!("{}/{}", args.symbol_a, args.symbol_b),
        period_days: args.days,
        entry_price,
        exit_price: final_price,
        range_lower: args.lower_price,
        range_upper: args.upper_price,
        initial_capital: args.capital,
        final_value: args.capital + result.summary.net_pnl,
        total_return,
        fee_earnings: result.summary.total_fees,
        impermanent_loss: result.summary.final_il_pct,
        vs_hodl,
        time_in_range: result.summary.time_in_range_pct() * Decimal::from(100),
        max_drawdown: result.summary.max_drawdown_pct,
        rebalance_count: result.summary.rebalance_count,
        total_tx_costs: Decimal::from(result.summary.rebalance_count) * args.tx_cost,
        strategy: format!("{:?}", args.strategy),
        sharpe_ratio: calculate_sharpe(&result.pnl_history),
    })
}

/// Generates synthetic prices for testing.
fn generate_synthetic_prices(count: usize) -> Vec<Price> {
    use rand::RngExt;

    let mut rng = rand::rng();
    let mut price = 100.0_f64;
    let mut prices = Vec::with_capacity(count);

    for _ in 0..count {
        prices.push(Price::new(Decimal::from_f64(price).unwrap()));
        // Random walk with slight drift
        let change = rng.random_range(-0.02..0.02);
        price *= 1.0 + change;
        price = price.clamp(50.0, 200.0);
    }

    prices
}

/// Calculates Sharpe ratio from PnL history.
fn calculate_sharpe(pnl_history: &[Decimal]) -> Option<Decimal> {
    if pnl_history.len() < 2 {
        return None;
    }

    let returns: Vec<Decimal> = pnl_history.windows(2).map(|w| w[1] - w[0]).collect();

    if returns.is_empty() {
        return None;
    }

    let mean: Decimal = returns.iter().copied().sum::<Decimal>() / Decimal::from(returns.len());

    let variance: Decimal = returns
        .iter()
        .map(|r| {
            let diff = *r - mean;
            diff * diff
        })
        .sum::<Decimal>()
        / Decimal::from(returns.len());

    let std_dev = variance.to_string().parse::<f64>().ok()?.sqrt();

    if std_dev < 0.0001 {
        return None;
    }

    let sharpe = mean / Decimal::from_f64(std_dev)?;
    Some(sharpe)
}

/// Prints backtest report in CSV format.
fn print_csv_backtest(report: &BacktestReport) {
    println!("metric,value");
    println!("pair,{}", report.pair);
    println!("period_days,{}", report.period_days);
    println!("entry_price,{}", report.entry_price);
    println!("exit_price,{}", report.exit_price);
    println!("initial_capital,{}", report.initial_capital);
    println!("final_value,{}", report.final_value);
    println!("total_return_pct,{}", report.total_return);
    println!("fee_earnings,{}", report.fee_earnings);
    println!("impermanent_loss,{}", report.impermanent_loss);
    println!("vs_hodl,{}", report.vs_hodl);
    println!("time_in_range_pct,{}", report.time_in_range);
    println!("max_drawdown,{}", report.max_drawdown);
    println!("rebalance_count,{}", report.rebalance_count);
    println!("total_tx_costs,{}", report.total_tx_costs);
    println!("strategy,{}", report.strategy);
    if let Some(sharpe) = report.sharpe_ratio {
        println!("sharpe_ratio,{}", sharpe);
    }
}
