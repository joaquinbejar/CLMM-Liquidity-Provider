//! Command Line Interface for the CLMM Liquidity Provider.
use anyhow::Result;
use clap::{Parser, Subcommand};
use clmm_lp_data::{MarketDataProvider, providers::BirdeyeProvider};
use clmm_lp_domain::entities::position::Position;
use clmm_lp_domain::entities::token::Token;
use clmm_lp_domain::enums::PositionStatus;
use clmm_lp_domain::value_objects::amount::Amount;
use clmm_lp_domain::value_objects::price::Price;
use clmm_lp_domain::value_objects::price_range::PriceRange;
use clmm_lp_simulation::engine::SimulationEngine;
use clmm_lp_simulation::liquidity::ConstantLiquidity;
use clmm_lp_simulation::price_path::HistoricalPricePath;
use clmm_lp_simulation::volume::ConstantVolume;
use dotenv::dotenv;
use primitive_types::U256;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "amm-cli")]
#[command(about = "CLMM Liquidity Provider Strategy Optimizer CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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

        /// Initial capital in USD (approx)
        #[arg(long, default_value_t = 1000.0)]
        capital: f64,
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

            println!("📡 Initializing Birdeye Provider...");
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

            println!(
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
            println!(
                "{:<20} | {:<10} | {:<10} | {:<10} | {:<10}",
                "Time", "Open", "High", "Low", "Close"
            );
            println!("{}", "-".repeat(70));

            for candle in candles {
                let datetime = chrono::DateTime::from_timestamp(candle.start_timestamp as i64, 0)
                    .unwrap_or_default();
                println!(
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
            let price_path = HistoricalPricePath::new(prices.clone());

            // Prepare Position
            let liquidity_amount = (*capital as u128) * 10; // Simplified proxy for liquidity
            let position = Position {
                id: clmm_lp_domain::entities::position::PositionId(Uuid::new_v4()),
                pool_address: "sim-pool".to_string(),
                owner_address: "user".to_string(),
                liquidity_amount,
                deposited_amount_a: Amount::new(U256::zero(), 9),
                deposited_amount_b: Amount::new(U256::zero(), 6),
                current_amount_a: Amount::new(U256::zero(), 9),
                current_amount_b: Amount::new(U256::zero(), 6),
                unclaimed_fees_a: Amount::new(U256::zero(), 9),
                unclaimed_fees_b: Amount::new(U256::zero(), 6),
                range: Some(PriceRange::new(
                    Price::new(Decimal::from_f64(*lower).unwrap()),
                    Price::new(Decimal::from_f64(*upper).unwrap()),
                )),
                opened_at: start_time,
                status: PositionStatus::Open,
            };

            // Models
            let volume = ConstantVolume {
                amount: Amount::new(U256::from(1_000_000_000_000u64), 6), // 1M USDC vol per step
            };
            let liquidity_model = ConstantLiquidity::new(liquidity_amount * 100); // 1% share
            let fee_rate = Decimal::from_f64(0.003).unwrap(); // 0.3%

            let mut engine = SimulationEngine::new(
                position,
                price_path,
                volume,
                liquidity_model,
                fee_rate,
                prices.len(),
            );

            println!("🚀 Running simulation over {} steps...", prices.len());
            let result = engine.run();

            println!("\n📊 Backtest Results");
            println!("════════════════════════════════════");
            println!("Initial Capital: ${:.2}", capital);
            println!("Final Value:     ${:.2}", result.final_position_value);
            println!("Net PnL:         ${:.2}", result.net_pnl);
            println!("Fees Earned:     ${:.2}", result.total_fees_earned);
            println!("Impermanent Loss:${:.2}", result.total_il);
            println!(
                "Time in Range:   {:.1}%",
                result.time_in_range_percentage * Decimal::from(100)
            );
            println!("════════════════════════════════════");
        }
    }

    Ok(())
}
