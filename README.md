# CLMM Liquidity Provider Strategy Optimizer

A high-performance, modular Rust framework for **backtesting, simulating, and optimizing** concentrated liquidity (CLMM) positions on Solana AMMs like **Orca Whirlpools** and **Raydium**.

This project aims to help Liquidity Providers (LPs) make data-driven decisions by answering questions like:
- "What is the optimal price range for my position?"
- "How much fee revenue can I expect given current volatility?"
- "What is my Value at Risk (VaR) due to Impermanent Loss?"

## 🏗 Architecture

The project follows a **Domain-Driven Design (DDD)** approach with a clean separation of concerns:

### 1. Domain Layer (`crates/domain`)
Pure Rust implementation of the core business logic and mathematical models.
- **Entities**: `Pool`, `Position`, `Token`.
- **Math**: Constant Product & Concentrated Liquidity formulas ($L$, $\sqrt{P}$, amount deltas), Impermanent Loss calculations.
- **Value Objects**: `Price`, `PriceRange`, `Amount`.

### 2. Simulation Engine (`crates/simulation`)
A powerful engine to project the future performance of LP positions.
- **Monte Carlo Simulation**: Generates thousands of stochastic price paths (Geometric Brownian Motion) to estimate risk/reward.
- **Deterministic Backtesting**: Replay historical price actions to validate strategies.
- **Metrics**: PnL, Fees Earned, Impermanent Loss, Time in Range, Sharpe Ratio.

### 3. Optimization Engine (`crates/optimization`)
Algorithms to find the best strategy parameters.
- **Range Optimizer**: Iteratively evaluates different price ranges (e.g., +/- 2%, 5%, 10%) to maximize specific objectives (Net PnL, Fees, Risk-Adjusted Return).
- **Objective Functions**: Flexible traits to define what "success" looks like.

### 4. Protocols & Infrastructure (`crates/protocols`)
Adapters for the Solana blockchain.
- **Solana RPC Client**: Fetches live on-chain state.
- **Parsers**: Deserializes raw account data (Borsh) into domain entities (e.g., parsing Orca Whirlpool accounts).

### 5. Data Layer (`crates/data`)
Providers for external market data.
- **Market Data**: Fetches historical price candles (OHLCV) and volume data for backtesting and volatility estimation.

## 🚀 Getting Started

### Prerequisites
- Rust (latest stable)
- Solana CLI (optional, for local validator)

### Building
```bash
cargo build --workspace
```

### Testing
Run the extensive unit test suite across all crates:
```bash
cargo test --workspace
```

## 🗺 Roadmap
- [x] **Domain Core**: Math & Entities.
- [x] **Simulation Engine**: GBM & Deterministic runners.
- [x] **Optimization**: Range selection logic.
- [x] **Protocol Adapters**: Basic Solana RPC & Orca structures.
- [ ] **Market Data**: Integration with Birdeye/Jupiter APIs.
- [ ] **CLI**: Command-line interface for running optimizations.
- [ ] **Live Execution**: integration with wallets to execute strategies (future).

## 📄 License
MIT
