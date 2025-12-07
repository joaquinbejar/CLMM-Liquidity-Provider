//! Command Line Interface for the CLMM Liquidity Provider.
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use clmm_lp_data::prelude::*;
use clmm_lp_domain::prelude::*;
use clmm_lp_optimization::prelude::*;
use clmm_lp_simulation::prelude::*;
use dotenv::dotenv;
use prettytable::{Table, row};
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
    /// Database management commands
    Db {
        #[command(subcommand)]
        action: DbAction,
    },
    /// Analyze a token pair's historical data
    Analyze {
        /// Token A Symbol (e.g., SOL)
        #[arg(short, long, default_value = "SOL")]
        symbol_a: String,

        /// Token A Mint Address
        #[arg(long, default_value = "So11111111111111111111111111111111111111112")]
        mint_a: String,

        /// Days of history to analyze
        #[arg(short, long, default_value_t = 30)]
        days: u64,
    },
}

/// Database management actions.
#[derive(Subcommand)]
enum DbAction {
    /// Initialize the database with migrations
    Init,
    /// Show database connection status
    Status,
    /// List recent simulations
    ListSimulations {
        /// Maximum number of results
        #[arg(short, long, default_value_t = 10)]
        limit: i64,
    },
    /// List recent optimizations
    ListOptimizations {
        /// Maximum number of results
        #[arg(short, long, default_value_t = 10)]
        limit: i64,
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

            println!("✅ Fetched {} candles:", candles.len());
            println!();

            let mut table = Table::new();
            table.add_row(row!["Time", "Open", "High", "Low", "Close"]);

            for candle in candles {
                let datetime = chrono::DateTime::from_timestamp(candle.start_timestamp as i64, 0)
                    .unwrap_or_default();
                table.add_row(row![
                    datetime.format("%Y-%m-%d %H:%M"),
                    format!("{:.4}", candle.open.value),
                    format!("{:.4}", candle.high.value),
                    format!("{:.4}", candle.low.value),
                    format!("{:.4}", candle.close.value)
                ]);
            }
            table.printstd();
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
        Commands::Db { action } => {
            let database_url = env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/clmm_lp".to_string());

            match action {
                DbAction::Init => {
                    println!("🔧 Initializing database...");
                    let db = Database::connect(&database_url).await?;
                    db.migrate().await?;
                    println!("✅ Database initialized successfully!");
                }
                DbAction::Status => {
                    println!("🔍 Checking database connection...");
                    match Database::connect(&database_url).await {
                        Ok(_) => {
                            println!("✅ Connected to database: {}", database_url);
                        }
                        Err(e) => {
                            println!("❌ Failed to connect: {}", e);
                        }
                    }
                }
                DbAction::ListSimulations { limit } => {
                    let db = Database::connect(&database_url).await?;
                    let simulations = db.simulations().find_recent(*limit).await?;

                    if simulations.is_empty() {
                        println!("No simulations found.");
                    } else {
                        println!("📊 Recent Simulations:");
                        println!();
                        let mut table = Table::new();
                        table.add_row(row!["ID", "Strategy", "Capital", "Range", "Created"]);
                        for sim in simulations {
                            table.add_row(row![
                                sim.id.to_string()[..8].to_string(),
                                sim.strategy_type,
                                format!("${:.2}", sim.initial_capital),
                                format!("${:.2} - ${:.2}", sim.lower_price, sim.upper_price),
                                sim.created_at.format("%Y-%m-%d %H:%M")
                            ]);
                        }
                        table.printstd();
                    }
                }
                DbAction::ListOptimizations { limit } => {
                    let db = Database::connect(&database_url).await?;
                    let optimizations = db.simulations().find_recent_optimizations(*limit).await?;

                    if optimizations.is_empty() {
                        println!("No optimizations found.");
                    } else {
                        println!("🎯 Recent Optimizations:");
                        println!();
                        let mut table = Table::new();
                        table.add_row(row!["ID", "Objective", "Range", "Expected PnL", "Created"]);
                        for opt in optimizations {
                            table.add_row(row![
                                opt.id.to_string()[..8].to_string(),
                                opt.objective_type,
                                format!(
                                    "${:.2} - ${:.2}",
                                    opt.recommended_lower, opt.recommended_upper
                                ),
                                format!("${:+.4}", opt.expected_pnl),
                                opt.created_at.format("%Y-%m-%d %H:%M")
                            ]);
                        }
                        table.printstd();
                    }
                }
            }
        }
        Commands::Analyze {
            symbol_a,
            mint_a,
            days,
        } => {
            let api_key = env::var("BIRDEYE_API_KEY")
                .expect("BIRDEYE_API_KEY must be set in .env or environment");

            println!("📊 Analyzing {}/USDC over {} days...", symbol_a, days);
            println!();

            let provider = BirdeyeProvider::new(api_key);

            let token_a = Token::new(mint_a, symbol_a, 9, symbol_a);
            let token_b = Token::new(
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "USDC",
                6,
                "USD Coin",
            );

            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let start_time = now - (days * 24 * 3600);

            let candles = provider
                .get_price_history(&token_a, &token_b, start_time, now, 3600)
                .await?;

            if candles.is_empty() {
                println!("❌ No data available for the specified period.");
                return Ok(());
            }

            // Calculate statistics
            let prices: Vec<f64> = candles
                .iter()
                .filter_map(|c| c.close.value.to_f64())
                .collect();

            let current_price = prices.last().copied().unwrap_or(0.0);
            let first_price = prices.first().copied().unwrap_or(0.0);
            let max_price = prices.iter().copied().fold(f64::MIN, f64::max);
            let min_price = prices.iter().copied().fold(f64::MAX, f64::min);
            let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

            let price_change = if first_price > 0.0 {
                (current_price - first_price) / first_price * 100.0
            } else {
                0.0
            };

            let volatility = calculate_volatility(&prices);
            let volatility_daily = volatility / (365.0_f64).sqrt();

            // Calculate volume stats
            let total_volume: f64 = candles
                .iter()
                .map(|c| c.volume_token_a.to_decimal().to_f64().unwrap_or(0.0))
                .sum();
            let avg_hourly_volume = total_volume / candles.len() as f64;

            // Print analysis report
            println!("🎯 ANALYSIS RESULTS: {}/USDC", symbol_a);
            println!();

            // Price Statistics Table
            let mut price_table = Table::new();
            price_table.add_row(row!["PRICE STATISTICS", ""]);
            price_table.add_row(row!["Current Price", format!("${:.4}", current_price)]);
            price_table.add_row(row!["Period Start", format!("${:.4}", first_price)]);
            price_table.add_row(row!["Period High", format!("${:.4}", max_price)]);
            price_table.add_row(row!["Period Low", format!("${:.4}", min_price)]);
            price_table.add_row(row!["Average Price", format!("${:.4}", avg_price)]);
            price_table.add_row(row!["Price Change", format!("{:+.2}%", price_change)]);
            price_table.add_row(row![
                "Price Range",
                format!("${:.4} - ${:.4}", min_price, max_price)
            ]);
            price_table.printstd();

            println!();

            // Volatility Table
            let mut vol_table = Table::new();
            vol_table.add_row(row!["VOLATILITY METRICS", ""]);
            vol_table.add_row(row![
                "Annualized Volatility",
                format!("{:.1}%", volatility * 100.0)
            ]);
            vol_table.add_row(row![
                "Daily Volatility",
                format!("{:.2}%", volatility_daily * 100.0)
            ]);
            vol_table.add_row(row!["Data Points", format!("{} candles", candles.len())]);
            vol_table.printstd();

            println!();

            // Volume Table
            let mut volume_table = Table::new();
            volume_table.add_row(row!["VOLUME METRICS", ""]);
            volume_table.add_row(row![
                "Total Volume",
                format!("{:.2} {}", total_volume, symbol_a)
            ]);
            volume_table.add_row(row![
                "Avg Hourly Volume",
                format!("{:.2} {}", avg_hourly_volume, symbol_a)
            ]);
            volume_table.add_row(row![
                "Avg Daily Volume",
                format!("{:.2} {}", avg_hourly_volume * 24.0, symbol_a)
            ]);
            volume_table.printstd();

            println!();

            // Suggested ranges based on volatility
            let range_1x = current_price * volatility_daily;
            let range_2x = current_price * volatility_daily * 2.0;

            let mut suggest_table = Table::new();
            suggest_table.add_row(row!["SUGGESTED LP RANGES", ""]);
            suggest_table.add_row(row![
                "Conservative (1σ daily)",
                format!(
                    "${:.2} - ${:.2}",
                    current_price - range_1x,
                    current_price + range_1x
                )
            ]);
            suggest_table.add_row(row![
                "Moderate (2σ daily)",
                format!(
                    "${:.2} - ${:.2}",
                    current_price - range_2x,
                    current_price + range_2x
                )
            ]);
            suggest_table.add_row(row![
                "Wide (period range)",
                format!("${:.2} - ${:.2}", min_price * 0.95, max_price * 1.05)
            ]);
            suggest_table.printstd();

            println!();
            println!("💡 Tip: Use these ranges with the backtest command:");
            println!(
                "   clmm-lp-cli backtest --lower {:.2} --upper {:.2} --days {}",
                current_price - range_2x,
                current_price + range_2x,
                days
            );
            println!();
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

/// Prints a rich backtest report using prettytable.
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
    println!("📊 BACKTEST RESULTS: {}/USDC", symbol);
    println!("Period: {} days | Strategy: {:?}", days, strategy);
    println!();

    // Position Configuration Table
    let mut config_table = Table::new();
    config_table.add_row(row!["POSITION CONFIGURATION", ""]);
    config_table.add_row(row![
        "Price Range",
        format!("${:.2} - ${:.2}", lower, upper)
    ]);
    config_table.add_row(row!["Entry Price", format!("${:.4}", entry_price)]);
    config_table.add_row(row![
        "Final Price",
        format!("${:.4} ({:+.2}%)", final_price, price_change_pct)
    ]);
    config_table.add_row(row!["Initial Capital", format!("${:.2}", capital)]);
    config_table.printstd();

    println!();

    // Performance Metrics Table
    let mut perf_table = Table::new();
    perf_table.add_row(row!["PERFORMANCE METRICS", ""]);
    perf_table.add_row(row!["Final Value", format!("${:.2}", summary.final_value)]);
    perf_table.add_row(row![
        "Net PnL",
        format!("${:+.2} ({:+.2}%)", summary.final_pnl, return_pct)
    ]);
    perf_table.add_row(row!["Fees Earned", format!("${:.2}", summary.total_fees)]);
    perf_table.add_row(row![
        "Impermanent Loss",
        format!("{:.2}%", summary.final_il_pct * Decimal::from(100))
    ]);
    perf_table.printstd();

    println!();

    // Risk Metrics Table
    let mut risk_table = Table::new();
    risk_table.add_row(row!["RISK METRICS", ""]);
    risk_table.add_row(row![
        "Time in Range",
        format!("{:.1}%", summary.time_in_range_pct * Decimal::from(100))
    ]);
    risk_table.add_row(row![
        "Max Drawdown",
        format!("{:.2}%", summary.max_drawdown * Decimal::from(100))
    ]);
    risk_table.add_row(row![
        "Rebalances",
        format!(
            "{} (cost: ${:.2})",
            summary.rebalance_count, summary.total_rebalance_cost
        )
    ]);
    risk_table.printstd();

    println!();

    // Comparison Table
    let mut comp_table = Table::new();
    comp_table.add_row(row!["COMPARISON vs HODL", ""]);
    comp_table.add_row(row!["HODL Value", format!("${:.2}", summary.hodl_value)]);
    comp_table.add_row(row![
        "LP vs HODL",
        format!("${:+.2} ({:+.2}%)", summary.vs_hodl, vs_hodl_pct)
    ]);
    comp_table.printstd();

    println!();
}

/// Prints optimization results using prettytable.
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
    println!("🎯 OPTIMIZATION RESULTS: {}/USDC", symbol);
    println!();

    // Market Conditions Table
    let mut market_table = Table::new();
    market_table.add_row(row!["MARKET CONDITIONS", ""]);
    market_table.add_row(row!["Current Price", format!("${:.4}", current_price)]);
    market_table.add_row(row![
        "Volatility (annualized)",
        format!("{:.1}%", volatility * 100.0)
    ]);
    market_table.add_row(row!["Capital", format!("${:.2}", capital)]);
    market_table.printstd();

    println!();

    // Recommended Range Table
    let mut range_table = Table::new();
    range_table.add_row(row!["RECOMMENDED RANGE", ""]);
    range_table.add_row(row!["Lower Bound", format!("${:.4}", lower)]);
    range_table.add_row(row!["Upper Bound", format!("${:.4}", upper)]);
    range_table.add_row(row!["Range Width", format!("{}%", width_pct)]);
    range_table.printstd();

    println!();

    // Expected Performance Table
    let mut perf_table = Table::new();
    perf_table.add_row(row!["EXPECTED PERFORMANCE", ""]);
    perf_table.add_row(row![
        "Expected PnL",
        format!("${:+.4}", result.expected_pnl)
    ]);
    perf_table.add_row(row![
        "Expected Fees",
        format!("${:.4}", result.expected_fees)
    ]);
    perf_table.add_row(row!["Expected IL", format!("${:.4}", result.expected_il)]);
    if let Some(sharpe) = result.sharpe_ratio {
        perf_table.add_row(row!["Sharpe Ratio", format!("{:.2}", sharpe)]);
    }
    perf_table.printstd();

    println!();
    println!("💡 Tip: Use these bounds with the backtest command:");
    println!(
        "   clmm-lp-cli backtest --lower {:.2} --upper {:.2}",
        lower, upper
    );
    println!();
}
