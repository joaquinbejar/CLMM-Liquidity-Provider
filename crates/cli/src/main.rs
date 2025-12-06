//! Command Line Interface for the CLMM Liquidity Provider.
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use clmm_lp_data::prelude::*;
use clmm_lp_domain::prelude::*;
use clmm_lp_optimization::prelude::*;
use clmm_lp_simulation::prelude::*;
use dotenv::dotenv;
use primitive_types::U256;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "clmm-lp-cli")]
#[command(about = "CLMM Liquidity Provider Strategy Optimizer CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Optimization objective for range optimization.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum OptimizationObjectiveArg {
    /// Maximize net PnL (fees - IL)
    Pnl,
    /// Maximize fees earned
    Fees,
    /// Maximize Sharpe ratio (risk-adjusted returns)
    Sharpe,
}

/// Rebalancing strategy for backtest.
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum StrategyArg {
    /// No rebalancing - hold initial range
    #[default]
    Static,
    /// Rebalance at fixed intervals
    Periodic,
    /// Rebalance when price moves beyond threshold
    Threshold,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch recent market data
    MarketData {
        /// Token A Symbol (e.g., SOL)
        #[arg(short, long, default_value = "SOL")]
        symbol_a: String,

        /// Token A Mint Address
        #[arg(long, default_value = "So11111111111111111111111111111111111111112")]
        mint_a: String,

        /// Hours of history to fetch
        #[arg(short, long, default_value_t = 24)]
        hours: u64,
    },
    /// Run a backtest on historical data
    Backtest {
        /// Token A Symbol (e.g., SOL)
        #[arg(short, long, default_value = "SOL")]
        symbol_a: String,

        /// Token A Mint Address
        #[arg(long, default_value = "So11111111111111111111111111111111111111112")]
        mint_a: String,

        /// Days of history to backtest
        #[arg(short, long, default_value_t = 30)]
        days: u64,

        /// Lower price bound
        #[arg(long)]
        lower: f64,

        /// Upper price bound
        #[arg(long)]
        upper: f64,

        /// Initial capital in USD
        #[arg(long, default_value_t = 1000.0)]
        capital: f64,

        /// Rebalancing strategy
        #[arg(long, value_enum, default_value_t = StrategyArg::Static)]
        strategy: StrategyArg,

        /// Rebalance interval in hours (for periodic strategy)
        #[arg(long, default_value_t = 24)]
        rebalance_interval: u64,

        /// Price threshold percentage for rebalance (for threshold strategy)
        #[arg(long, default_value_t = 0.05)]
        threshold_pct: f64,

        /// Transaction cost per rebalance in USD
        #[arg(long, default_value_t = 1.0)]
        tx_cost: f64,
    },
    /// Optimize price range for LP position
    Optimize {
        /// Token A Symbol (e.g., SOL)
        #[arg(short, long, default_value = "SOL")]
        symbol_a: String,

        /// Token A Mint Address
        #[arg(long, default_value = "So11111111111111111111111111111111111111112")]
        mint_a: String,

        /// Days of history to analyze for volatility
        #[arg(short, long, default_value_t = 30)]
        days: u64,

        /// Initial capital in USD
        #[arg(long, default_value_t = 1000.0)]
        capital: f64,

        /// Optimization objective
        #[arg(long, value_enum, default_value_t = OptimizationObjectiveArg::Pnl)]
        objective: OptimizationObjectiveArg,

        /// Number of Monte Carlo iterations
        #[arg(long, default_value_t = 100)]
        iterations: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::MarketData {
            symbol_a,
            mint_a,
            hours,
        } => {
            let api_key = env::var("BIRDEYE_API_KEY")
                .expect("BIRDEYE_API_KEY must be set in .env or environment");

            info!("📡 Initializing Birdeye Provider...");
            let provider = BirdeyeProvider::new(api_key);

            // Define Tokens (Token B assumed USDC for this demo)
            let token_a = Token::new(mint_a, symbol_a, 9, symbol_a);
            let token_b = Token::new(
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "USDC",
                6,
                "USD Coin",
            );

            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let start_time = now - (hours * 3600);

            info!(
                "🔍 Fetching data for {}/USDC from {} to {}...",
                symbol_a, start_time, now
            );

            // Fetch 1-hour candles
            let candles = provider
                .get_price_history(
                    &token_a, &token_b, start_time, now, 3600, // 1h resolution
                )
                .await?;

            info!("✅ Fetched {} candles:", candles.len());
            info!(
                "{:<20} | {:<10} | {:<10} | {:<10} | {:<10}",
                "Time", "Open", "High", "Low", "Close"
            );
            info!("{}", "-".repeat(70));

            for candle in candles {
                let datetime = chrono::DateTime::from_timestamp(candle.start_timestamp as i64, 0)
                    .unwrap_or_default();
                info!(
                    "{:<20} | {:<10.4} | {:<10.4} | {:<10.4} | {:<10.4}",
                    datetime.format("%Y-%m-%d %H:%M"),
                    candle.open.value,
                    candle.high.value,
                    candle.low.value,
                    candle.close.value
                );
            }
        }
        Commands::Backtest {
            symbol_a,
            mint_a,
            days,
            lower,
            upper,
            capital,
            strategy,
            rebalance_interval,
            threshold_pct,
            tx_cost,
        } => {
            let api_key = env::var("BIRDEYE_API_KEY")
                .expect("BIRDEYE_API_KEY must be set in .env or environment");

            println!("📡 Initializing Backtest Engine...");
            let provider = BirdeyeProvider::new(api_key);

            // Define Tokens
            let token_a = Token::new(mint_a, symbol_a, 9, symbol_a);
            let token_b = Token::new(
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "USDC",
                6,
                "USD Coin",
            );

            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let start_time = now - (days * 24 * 3600);

            println!(
                "🔍 Fetching historical data for {}/USDC ({} days)...",
                symbol_a, days
            );

            let candles = provider
                .get_price_history(&token_a, &token_b, start_time, now, 3600) // 1h resolution
                .await?;

            if candles.is_empty() {
                println!("❌ No data found for the specified period.");
                return Ok(());
            }

            // Prepare Price Path
            let prices: Vec<Price> = candles.iter().map(|c| c.close).collect();
            let entry_price = prices.first().cloned().unwrap_or(Price::new(Decimal::ONE));
            let final_price = prices.last().cloned().unwrap_or(entry_price);

            // Setup position tracker
            let initial_range = PriceRange::new(
                Price::new(Decimal::from_f64(*lower).unwrap()),
                Price::new(Decimal::from_f64(*upper).unwrap()),
            );
            let capital_dec = Decimal::from_f64(*capital).unwrap();
            let tx_cost_dec = Decimal::from_f64(*tx_cost).unwrap();

            let mut tracker =
                PositionTracker::new(capital_dec, entry_price, initial_range, tx_cost_dec);

            // Setup volume and liquidity models
            let mut volume_model = ConstantVolume {
                amount: Amount::new(U256::from(1_000_000_000_000u64), 6), // 1M USDC vol per step
            };
            let liquidity_amount = (*capital as u128) * 10;
            let global_liquidity = liquidity_amount * 100; // 1% share
            let fee_rate = Decimal::from_f64(0.003).unwrap();

            println!(
                "🚀 Running backtest with {:?} strategy over {} steps...",
                strategy,
                prices.len()
            );

            // Run simulation with strategy
            let range_width_pct =
                Decimal::from_f64((*upper - *lower) / ((*upper + *lower) / 2.0)).unwrap();

            for price in &prices {
                // Calculate fees for this step
                let in_range = price.value >= tracker.current_range.lower_price.value
                    && price.value <= tracker.current_range.upper_price.value;

                let step_fees = if in_range {
                    let vol = volume_model.next_volume().to_decimal();
                    let fee_share =
                        Decimal::from(liquidity_amount) / Decimal::from(global_liquidity);
                    vol * fee_share * fee_rate
                } else {
                    Decimal::ZERO
                };

                // Apply strategy
                match strategy {
                    StrategyArg::Static => {
                        let strat = StaticRange::new();
                        tracker.record_step(*price, step_fees, Some(&strat));
                    }
                    StrategyArg::Periodic => {
                        let strat = PeriodicRebalance::new(*rebalance_interval, range_width_pct);
                        tracker.record_step(*price, step_fees, Some(&strat));
                    }
                    StrategyArg::Threshold => {
                        let strat = ThresholdRebalance::new(
                            Decimal::from_f64(*threshold_pct).unwrap(),
                            range_width_pct,
                        );
                        tracker.record_step(*price, step_fees, Some(&strat));
                    }
                }
            }

            // Get summary
            let summary = tracker.summary();

            // Print rich report
            print_backtest_report(
                symbol_a,
                *days,
                *capital,
                entry_price.value,
                final_price.value,
                *lower,
                *upper,
                &summary,
                *strategy,
            );
        }
        Commands::Optimize {
            symbol_a,
            mint_a,
            days,
            capital,
            objective,
            iterations,
        } => {
            let api_key = env::var("BIRDEYE_API_KEY")
                .expect("BIRDEYE_API_KEY must be set in .env or environment");

            println!("📡 Initializing Optimizer...");
            let provider = BirdeyeProvider::new(api_key);

            // Define Tokens
            let token_a = Token::new(mint_a, symbol_a, 9, symbol_a);
            let token_b = Token::new(
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "USDC",
                6,
                "USD Coin",
            );

            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let start_time = now - (days * 24 * 3600);

            println!(
                "🔍 Fetching historical data for {}/USDC ({} days) to estimate volatility...",
                symbol_a, days
            );

            let candles = provider
                .get_price_history(&token_a, &token_b, start_time, now, 3600)
                .await?;

            if candles.is_empty() {
                println!("❌ No data found for the specified period.");
                return Ok(());
            }

            // Calculate volatility from historical data
            let prices: Vec<f64> = candles
                .iter()
                .map(|c| c.close.value.to_f64().unwrap_or(0.0))
                .collect();

            let volatility = calculate_volatility(&prices);
            let current_price = *prices.last().unwrap_or(&100.0);
            let current_price_dec = Decimal::from_f64(current_price).unwrap();

            println!("📊 Market Analysis:");
            println!("   Current Price: ${:.4}", current_price);
            println!("   Volatility (annualized): {:.1}%", volatility * 100.0);
            println!();

            // Setup optimizer
            let optimizer = RangeOptimizer::new(*iterations, 30, 1.0 / 365.0);

            let base_position = Position {
                id: clmm_lp_domain::entities::position::PositionId(Uuid::new_v4()),
                pool_address: "opt-pool".to_string(),
                owner_address: "user".to_string(),
                liquidity_amount: 0,
                deposited_amount_a: Amount::new(U256::zero(), 9),
                deposited_amount_b: Amount::new(U256::zero(), 6),
                current_amount_a: Amount::new(U256::zero(), 9),
                current_amount_b: Amount::new(U256::zero(), 6),
                unclaimed_fees_a: Amount::new(U256::zero(), 9),
                unclaimed_fees_b: Amount::new(U256::zero(), 6),
                range: None,
                opened_at: now,
                status: PositionStatus::Open,
            };

            let volume = ConstantVolume {
                amount: Amount::new(U256::from(1_000_000_000_000u64), 6),
            };
            let pool_liquidity = (*capital as u128) * 1000;
            let fee_rate = Decimal::from_f64(0.003).unwrap();

            println!(
                "🔄 Running optimization with {:?} objective ({} iterations)...",
                objective, iterations
            );

            let result = match objective {
                OptimizationObjectiveArg::Pnl => optimizer.optimize(
                    base_position,
                    current_price_dec,
                    volatility,
                    0.0,
                    volume,
                    pool_liquidity,
                    fee_rate,
                    MaximizeNetPnL,
                ),
                OptimizationObjectiveArg::Fees => optimizer.optimize(
                    base_position,
                    current_price_dec,
                    volatility,
                    0.0,
                    volume,
                    pool_liquidity,
                    fee_rate,
                    MaximizeFees,
                ),
                OptimizationObjectiveArg::Sharpe => optimizer.optimize(
                    base_position,
                    current_price_dec,
                    volatility,
                    0.0,
                    volume,
                    pool_liquidity,
                    fee_rate,
                    MaximizeSharpeRatio::new(Decimal::from_f64(0.05).unwrap()),
                ),
            };

            // Print optimization results
            print_optimization_report(symbol_a, current_price, volatility, *capital, &result);
        }
    }

    Ok(())
}

/// Calculates annualized volatility from price series.
fn calculate_volatility(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }

    // Calculate log returns
    let returns: Vec<f64> = prices.windows(2).map(|w| (w[1] / w[0]).ln()).collect();

    if returns.is_empty() {
        return 0.0;
    }

    // Calculate standard deviation
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
    let std_dev = variance.sqrt();

    // Annualize (assuming hourly data, ~8760 hours/year)
    std_dev * (8760.0_f64).sqrt()
}

/// Prints a rich backtest report.
#[allow(clippy::too_many_arguments)]
fn print_backtest_report(
    symbol: &str,
    days: u64,
    capital: f64,
    entry_price: Decimal,
    final_price: Decimal,
    lower: f64,
    upper: f64,
    summary: &TrackerSummary,
    strategy: StrategyArg,
) {
    let price_change_pct =
        ((final_price - entry_price) / entry_price * Decimal::from(100)).round_dp(2);
    let return_pct =
        (summary.final_pnl / Decimal::from_f64(capital).unwrap() * Decimal::from(100)).round_dp(2);
    let vs_hodl_pct = if summary.hodl_value != Decimal::ZERO {
        (summary.vs_hodl / summary.hodl_value * Decimal::from(100)).round_dp(2)
    } else {
        Decimal::ZERO
    };

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!(
        "║              📊 BACKTEST RESULTS: {}/USDC                   ║",
        symbol
    );
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ Period: {} days | Strategy: {:?}", days, strategy);
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ POSITION CONFIGURATION                                       ║");
    println!("║   Price Range: ${:.2} - ${:.2}", lower, upper);
    println!("║   Entry Price: ${:.4}", entry_price);
    println!(
        "║   Final Price: ${:.4} ({:+.2}%)",
        final_price, price_change_pct
    );
    println!("║   Initial Capital: ${:.2}", capital);
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ PERFORMANCE METRICS                                          ║");
    println!("║   Final Value:      ${:.2}", summary.final_value);
    println!(
        "║   Net PnL:          ${:+.2} ({:+.2}%)",
        summary.final_pnl, return_pct
    );
    println!("║   Fees Earned:      ${:.2}", summary.total_fees);
    println!(
        "║   Impermanent Loss: {:.2}%",
        summary.final_il_pct * Decimal::from(100)
    );
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ RISK METRICS                                                 ║");
    println!(
        "║   Time in Range:    {:.1}%",
        summary.time_in_range_pct * Decimal::from(100)
    );
    println!(
        "║   Max Drawdown:     {:.2}%",
        summary.max_drawdown * Decimal::from(100)
    );
    println!(
        "║   Rebalances:       {} (cost: ${:.2})",
        summary.rebalance_count, summary.total_rebalance_cost
    );
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ COMPARISON                                                   ║");
    println!("║   HODL Value:       ${:.2}", summary.hodl_value);
    println!(
        "║   vs HODL:          ${:+.2} ({:+.2}%)",
        summary.vs_hodl, vs_hodl_pct
    );
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
}

/// Prints optimization results.
fn print_optimization_report(
    symbol: &str,
    current_price: f64,
    volatility: f64,
    capital: f64,
    result: &OptimizationResult,
) {
    let lower = result.recommended_range.lower_price.value;
    let upper = result.recommended_range.upper_price.value;
    let width_pct = ((upper - lower) / Decimal::from_f64(current_price).unwrap()
        * Decimal::from(100))
    .round_dp(1);

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!(
        "║           🎯 OPTIMIZATION RESULTS: {}/USDC                  ║",
        symbol
    );
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ MARKET CONDITIONS                                            ║");
    println!("║   Current Price:    ${:.4}", current_price);
    println!(
        "║   Volatility:       {:.1}% (annualized)",
        volatility * 100.0
    );
    println!("║   Capital:          ${:.2}", capital);
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ RECOMMENDED RANGE                                            ║");
    println!("║   Lower Bound:      ${:.4}", lower);
    println!("║   Upper Bound:      ${:.4}", upper);
    println!("║   Range Width:      {}%", width_pct);
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ EXPECTED PERFORMANCE (per simulation period)                 ║");
    println!("║   Expected PnL:     ${:+.4}", result.expected_pnl);
    println!("║   Expected Fees:    ${:.4}", result.expected_fees);
    println!("║   Expected IL:      ${:.4}", result.expected_il);
    if let Some(sharpe) = result.sharpe_ratio {
        println!("║   Sharpe Ratio:     {:.2}", sharpe);
    }
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("💡 Tip: Use these bounds with the backtest command:");
    println!(
        "   clmm-lp-cli backtest --lower {:.2} --upper {:.2}",
        lower, upper
    );
    println!();
}
