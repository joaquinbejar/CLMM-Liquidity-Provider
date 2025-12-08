[![Dual License](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)
[![Stars](https://img.shields.io/github/stars/joaquinbejar/CLMM-Liquidity-Provider.svg)](https://github.com/joaquinbejar/CLMM-Liquidity-Provider/stargazers)
[![Issues](https://img.shields.io/github/issues/joaquinbejar/CLMM-Liquidity-Provider.svg)](https://github.com/joaquinbejar/CLMM-Liquidity-Provider/issues)
[![PRs](https://img.shields.io/github/issues-pr/joaquinbejar/CLMM-Liquidity-Provider.svg)](https://github.com/joaquinbejar/CLMM-Liquidity-Provider/pulls)
[![Build Status](https://img.shields.io/github/actions/workflow/status/joaquinbejar/CLMM-Liquidity-Provider/build.yml)](https://github.com/joaquinbejar/CLMM-Liquidity-Provider/actions)
[![Coverage](https://img.shields.io/codecov/c/github/joaquinbejar/CLMM-Liquidity-Provider)](https://codecov.io/gh/joaquinbejar/CLMM-Liquidity-Provider)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

### Crates

| Crate | Version | Docs |
|-------|---------|------|
| `clmm-lp-domain` | [![Crates.io](https://img.shields.io/crates/v/clmm-lp-domain.svg)](https://crates.io/crates/clmm-lp-domain) | [![Docs](https://docs.rs/clmm-lp-domain/badge.svg)](https://docs.rs/clmm-lp-domain) |
| `clmm-lp-simulation` | [![Crates.io](https://img.shields.io/crates/v/clmm-lp-simulation.svg)](https://crates.io/crates/clmm-lp-simulation) | [![Docs](https://docs.rs/clmm-lp-simulation/badge.svg)](https://docs.rs/clmm-lp-simulation) |
| `clmm-lp-optimization` | [![Crates.io](https://img.shields.io/crates/v/clmm-lp-optimization.svg)](https://crates.io/crates/clmm-lp-optimization) | [![Docs](https://docs.rs/clmm-lp-optimization/badge.svg)](https://docs.rs/clmm-lp-optimization) |
| `clmm-lp-protocols` | [![Crates.io](https://img.shields.io/crates/v/clmm-lp-protocols.svg)](https://crates.io/crates/clmm-lp-protocols) | [![Docs](https://docs.rs/clmm-lp-protocols/badge.svg)](https://docs.rs/clmm-lp-protocols) |
| `clmm-lp-execution` | [![Crates.io](https://img.shields.io/crates/v/clmm-lp-execution.svg)](https://crates.io/crates/clmm-lp-execution) | [![Docs](https://docs.rs/clmm-lp-execution/badge.svg)](https://docs.rs/clmm-lp-execution) |
| `clmm-lp-data` | [![Crates.io](https://img.shields.io/crates/v/clmm-lp-data.svg)](https://crates.io/crates/clmm-lp-data) | [![Docs](https://docs.rs/clmm-lp-data/badge.svg)](https://docs.rs/clmm-lp-data) |
| `clmm-lp-cli` | [![Crates.io](https://img.shields.io/crates/v/clmm-lp-cli.svg)](https://crates.io/crates/clmm-lp-cli) | [![Docs](https://docs.rs/clmm-lp-cli/badge.svg)](https://docs.rs/clmm-lp-cli) |
| `clmm-lp-api` | [![Crates.io](https://img.shields.io/crates/v/clmm-lp-api.svg)](https://crates.io/crates/clmm-lp-api) | [![Docs](https://docs.rs/clmm-lp-api/badge.svg)](https://docs.rs/clmm-lp-api) |

<div style="text-align: center;">
<img src="https://raw.githubusercontent.com/joaquinbejar/CLMM-Liquidity-Provider/main/doc/images/logo.png" alt="CLMM Liquidity Provider" style="width: 100%; height: 100%;">
</div>

# CLMM Liquidity Provider Strategy Optimizer - Solana

A powerful, production-ready strategy optimization and execution engine for liquidity providers on Solana Concentrated Liquidity Market Makers (CLMMs). This project supports **Orca Whirlpools**, **Raydium CLMM**, and **Meteora DLMM**.

Built for market makers and sophisticated LPs who need to:
- Analyze pools and calculate impermanent loss (IL) in real-time
- Simulate positions over historical data with multiple rebalancing strategies
- Execute automated rebalancing based on volatility, volume, and asset correlation
- Monitor positions with real-time alerts and notifications
- Access everything via CLI, REST API, or Web Dashboard

## 🚀 Value Proposition

DeFi market making requires more than just chasing high APY. This project solves three critical problems:
1.  **Risk Quantification**: Precise calculation of impermanent loss risks before entering positions.
2.  **Range Optimization**: Finding the mathematical optimal tick ranges to maximize fee capture while minimizing IL.
3.  **Scenario Simulation**: Stress-testing positions against historical and Monte Carlo simulated market conditions.

---

## 🏗 Architecture

The project follows a **Domain-Driven Design (DDD)** approach, separated into modular crates within a cargo workspace.

```mermaid
graph TD
    subgraph "User Interfaces"
        User[User]
        Browser[Web Browser]
    end
    
    User --> CLI[CLI Crate]
    User --> API[API Crate]
    Browser --> Dashboard[Web Dashboard]
    Dashboard --> API
    
    subgraph "Application Layer"
        CLI --> Optimization[Optimization Crate]
        CLI --> Simulation[Simulation Crate]
        API --> Execution[Execution Crate]
        API --> Protocols[Protocols Crate]
    end
    
    subgraph "Core Domain"
        Optimization --> Domain[Domain Crate]
        Simulation --> Domain
        Execution --> Domain
        Protocols --> Domain
    end
    
    subgraph "Infrastructure"
        Simulation --> Data[Data Crate]
        Execution --> Protocols
        Protocols --> Data
        Data --> DB[(PostgreSQL)]
    end
    
    subgraph "External Services"
        Protocols -.-> |RPC| Solana[Solana Blockchain]
        Data -.-> |HTTP| Birdeye[Birdeye API]
        Data -.-> |HTTP| Jupiter[Jupiter API]
        API -.-> |WebSocket| Clients[WS Clients]
    end
```

### Module Overview

| Crate | Description |
|-------|-------------|
| **`clmm-lp-domain`** | Core entities, value objects, CLMM math, tick calculations, and IL formulas |
| **`clmm-lp-simulation`** | Backtesting engine, price path generators, rebalancing strategies |
| **`clmm-lp-optimization`** | Strategy optimization with Grid Search, objective functions (PnL, Sharpe, IL) |
| **`clmm-lp-protocols`** | Solana protocol adapters (Orca Whirlpools), RPC provider with failover |
| **`clmm-lp-execution`** | Live monitoring, PnL tracking, alerts, wallet management, strategy execution |
| **`clmm-lp-data`** | Data providers (Birdeye, Jupiter), caching, PostgreSQL repositories |
| **`clmm-lp-cli`** | CLI with analyze, backtest, optimize, monitor commands. Multiple output formats |
| **`clmm-lp-api`** | REST API with Swagger UI, JWT auth, WebSocket support |

### Web Dashboard

| Component | Description |
|-----------|-------------|
| **Dashboard** | Portfolio overview with charts and metrics |
| **Positions** | Position management with real-time updates |
| **Strategies** | Strategy configuration and control |
| **Pools** | Pool explorer with TVL, volume, APY |
| **Settings** | Configuration and preferences |

---

## 🛠 Tech Stack

### Backend (Rust)

| Category | Technology |
|----------|------------|
| **Language** | Rust 1.75+ (Edition 2024) |
| **Async Runtime** | Tokio |
| **Web Framework** | Axum |
| **Database** | PostgreSQL (SQLx) |
| **Blockchain** | `solana-sdk`, `solana-client` |
| **Math** | `rust_decimal`, fixed-point precision |
| **Serialization** | Serde, JSON |
| **Logging** | `tracing` with structured logs |
| **Testing** | `proptest` for property-based testing |
| **API Docs** | `utoipa` (OpenAPI/Swagger) |

### Frontend (TypeScript)

| Category | Technology |
|----------|------------|
| **Framework** | React 18 |
| **Build Tool** | Vite |
| **Styling** | TailwindCSS |
| **State Management** | React Query (TanStack) |
| **Routing** | React Router |
| **Charts** | Recharts |
| **Icons** | Lucide React |

---

## 📅 Roadmap & Status

The project is being built in incremental phases. **Current version: 0.1.1-alpha.2**

| Phase | Name | Status | Progress |
|-------|------|--------|----------|
| 0 | Foundation | ✅ Complete | 100% |
| 1 | Core Domain Models | ✅ Complete | 100% |
| 2 | Simulation Engine | ✅ Complete | 100% |
| 3 | Data Integration | ✅ Complete | 100% |
| 4 | Optimization Engine | ✅ Complete | 100% |
| 5 | CLI & Reporting | ✅ Complete | 100% |
| 6 | Database & Persistence | ✅ Complete | 100% |
| 7 | Blockchain Integration | ✅ Complete | 100% |
| 8 | Live Monitoring | ✅ Complete | 100% |
| 9 | Transaction Execution | ✅ Complete | 100% |
| 10 | Strategy Automation | ✅ Complete | 100% |
| 11 | REST API | ✅ Complete | 100% |
| 12 | Web Dashboard | ✅ Complete | 100% |
| 13 | Production Hardening | 🟡 In Progress | 40% |
| 14 | Advanced Features | 🟡 In Progress | 20% |

**Overall Progress: ~90%** (12/14 phases complete)

---

## ✨ Features

### Core Capabilities

- **CLMM Mathematics**: Full implementation of concentrated liquidity math (tick ↔ price, sqrt_price, liquidity calculations)
- **Impermanent Loss**: Precise IL calculation for concentrated positions with range boundaries
- **Backtesting**: Simulate LP positions against historical price data with multiple rebalancing strategies
- **Optimization**: Find optimal tick ranges using Grid Search with configurable objective functions
- **Multi-Protocol**: Support for Orca Whirlpools, Raydium CLMM (Meteora DLMM planned)

### Rebalancing Strategies

| Strategy | Description |
|----------|-------------|
| **Static** | Hold position without rebalancing |
| **Periodic** | Rebalance at fixed time intervals |
| **Threshold** | Rebalance when price moves beyond threshold |
| **IL Limit** | Rebalance when impermanent loss exceeds limit |

### Optimization Objectives

- **Maximize Net PnL** - Total return after fees and IL
- **Maximize Fee Earnings** - Focus on fee capture
- **Maximize Sharpe Ratio** - Risk-adjusted returns
- **Minimize IL** - Conservative IL minimization
- **Maximize Time in Range** - Optimize for range efficiency

### Live Monitoring

- **Position Tracking**: Real-time position state from on-chain
- **PnL Calculation**: Entry value, current value, fees, IL, net PnL, APY
- **Alert System**: Configurable rules for range exit, IL thresholds, PnL targets
- **Multi-Channel Notifications**: Console, file, webhook

### REST API

- **OpenAPI/Swagger**: Full API documentation at `/docs`
- **JWT Authentication**: Secure API access with role-based permissions
- **API Key Support**: Alternative authentication method
- **WebSocket**: Real-time position updates and alerts

### Web Dashboard

- **Portfolio Overview**: Total value, PnL, fees, impermanent loss
- **Position Management**: View, rebalance, collect fees, close positions
- **Strategy Control**: Create, configure, start/stop automated strategies
- **Pool Explorer**: Browse pools with TVL, volume, and APY metrics
- **Real-time Updates**: WebSocket integration for live data

---

## ⚡️ Quick Start

### Prerequisites

- **Rust**: 1.75+ (edition 2024)
- **Node.js**: 18+ (for web dashboard)
- **Make**: Build automation
- **Docker**: Optional, for PostgreSQL
- **PostgreSQL**: 14+ (optional, for persistence)

### Installation

```bash
# Clone the repository
git clone https://github.com/joaquinbejar/CLMM-Liquidity-Provider.git
cd CLMM-Liquidity-Provider

# Build the project
make build

# Run tests
make test

# Install CLI globally (optional)
cargo install --path crates/cli
```

### Common Commands

The project includes a comprehensive `Makefile` for common tasks:

```bash
# Build the project
make build

# Run all tests (Unit & Integration)
make test

# Format code
make fmt

# Run Clippy lints
make lint

# Fix linting issues automatically
make lint-fix

# Pre-push checks (format + lint + test)
make pre-push

# Generate Documentation
make doc-open
```

### Running the API Server

```bash
# Start the API server (default port 8080)
cargo run --bin clmm-lp-api

# Or with environment variables
RUST_LOG=info API_PORT=8080 cargo run --bin clmm-lp-api
```

The API will be available at:
- **REST API**: `http://localhost:8080/api/v1`
- **Swagger UI**: `http://localhost:8080/docs`
- **WebSocket**: `ws://localhost:8080/ws`

### Running the Web Dashboard

```bash
# Navigate to web directory
cd web

# Install dependencies
npm install

# Start development server
npm run dev
```

The dashboard will be available at `http://localhost:3000`.

> **Note**: The web dashboard requires the API server to be running on port 8080.

### CLI Usage

```bash
# Analyze a trading pair
clmm-lp-cli analyze --symbol-a SOL --symbol-b USDC --days 30

# Run a backtest with periodic rebalancing
clmm-lp-cli backtest --symbol-a SOL --symbol-b USDC \
  --capital 10000 --lower-price 80 --upper-price 120 \
  --strategy periodic --rebalance-interval 24

# Optimize range parameters
clmm-lp-cli optimize --symbol-a SOL --symbol-b USDC \
  --capital 10000 --objective sharpe

# Fetch and cache market data
clmm-lp-cli data fetch --symbol SOL --days 90

# Monitor a live position
clmm-lp-cli monitor --position <POSITION_ADDRESS> --interval 30
```

### Output Formats

The CLI supports multiple output formats:

- **Table**: Rich formatted tables (default)
- **JSON**: Machine-readable JSON output
- **CSV**: Spreadsheet-compatible export
- **HTML**: Web-ready reports
- **Markdown**: Documentation-friendly format

---

## 📂 Project Structure

```text
CLMM-Liquidity-Provider/
├── Cargo.toml              # Workspace configuration
├── Makefile                # Task runner
├── .env.example            # Environment variables template
├── crates/
│   ├── api/                # REST API (Axum)
│   │   ├── handlers/       # Request handlers
│   │   ├── models.rs       # API request/response models
│   │   ├── auth.rs         # JWT & API key authentication
│   │   ├── openapi.rs      # Swagger/OpenAPI documentation
│   │   └── websocket.rs    # WebSocket handlers
│   ├── cli/                # CLI application
│   │   ├── commands/       # analyze, backtest, optimize, data, monitor
│   │   └── output/         # table, chart, export modules
│   ├── data/               # Data layer
│   │   ├── providers/      # Birdeye, Jupiter API clients
│   │   ├── repositories/   # PostgreSQL repositories
│   │   ├── cache/          # In-memory and file caching
│   │   └── migrations/     # SQL migration files
│   ├── domain/             # Core business logic
│   │   ├── entities/       # Pool, Position, Token
│   │   ├── metrics/        # IL, fees, PnL calculations
│   │   └── math/           # Tick math, liquidity, sqrt_price
│   ├── execution/          # Live execution engine
│   │   ├── alerts/         # Alert rules and notifiers
│   │   ├── monitor/        # Position monitor, PnL tracker
│   │   ├── strategy/       # Decision engine, executor
│   │   ├── transaction/    # Builder, manager
│   │   └── wallet/         # Wallet management
│   ├── optimization/       # Strategy optimization
│   │   ├── grid_search.rs  # Grid search optimizer
│   │   └── objectives.rs   # Objective functions
│   ├── protocols/          # Blockchain adapters
│   │   ├── orca/           # Whirlpool reader, executor
│   │   ├── rpc/            # RPC provider with failover
│   │   └── events/         # Event fetcher and parser
│   └── simulation/         # Backtesting engine
│       ├── models/         # Price path, volume, liquidity
│       └── strategies/     # Static, Periodic, Threshold, IL Limit
├── web/                    # Web Dashboard (React)
│   ├── src/
│   │   ├── components/     # UI components
│   │   ├── pages/          # Page components
│   │   ├── lib/            # API client, utilities
│   │   └── hooks/          # React hooks
│   ├── package.json
│   └── vite.config.ts
└── doc/                    # Documentation
    ├── steps.md            # Implementation roadmap
    └── resume.md           # Project overview
```

## 🧪 Testing Strategy

We employ a rigorous testing strategy to ensure financial safety:

1. **Unit Tests**: For all mathematical formulas (Tick Math, IL)
2. **Property-Based Tests**: To verify invariants (e.g., "Liquidity can never be negative")
3. **Integration Tests**: Mocked RPC calls to verify protocol adapters
4. **Simulation Tests**: Monte Carlo runs to stress-test strategies
5. **End-to-End Tests**: Full workflow testing with test fixtures

```bash
# Run all tests
make test

# Run with coverage
make coverage

# Run specific crate tests
cargo test -p clmm-lp-domain
cargo test -p clmm-lp-simulation
```

---

## 🔧 Configuration

### Environment Variables

Create a `.env` file based on `.env.example`:

```bash
# Solana RPC
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_RPC_BACKUP_URL=https://solana-api.projectserum.com

# Database (optional)
DATABASE_URL=postgres://user:pass@localhost:5432/clmm_lp

# API Configuration
API_HOST=0.0.0.0
API_PORT=8080
JWT_SECRET=your-secret-key-change-in-production

# Data Providers
BIRDEYE_API_KEY=your-birdeye-api-key
JUPITER_API_URL=https://price.jup.ag/v4

# Logging
RUST_LOG=info,clmm_lp=debug
```

### Strategy Configuration

Strategies can be configured via JSON files:

```json
{
  "name": "SOL-USDC Rebalancer",
  "strategy_type": "threshold",
  "pool_address": "HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ",
  "parameters": {
    "rebalance_threshold_pct": 5.0,
    "max_il_pct": 2.0,
    "min_rebalance_interval_hours": 4,
    "range_width_pct": 10.0
  }
}
```

---

## 🔒 Security Considerations

- **Dry Run Mode**: Always test strategies in dry-run mode before live execution
- **API Keys**: Never commit API keys or secrets to version control
- **Wallet Security**: Use hardware wallets or secure key management for production
- **Rate Limiting**: Built-in rate limiting for RPC and API calls
- **Circuit Breaker**: Automatic strategy pause on excessive losses

---

## 📊 API Endpoints

### Health & Metrics

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/health` | Health check |
| GET | `/api/v1/metrics` | System metrics |

### Positions

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/positions` | List all positions |
| GET | `/api/v1/positions/:address` | Get position details |
| POST | `/api/v1/positions` | Open new position |
| DELETE | `/api/v1/positions/:address` | Close position |
| POST | `/api/v1/positions/:address/rebalance` | Rebalance position |
| POST | `/api/v1/positions/:address/collect` | Collect fees |

### Strategies

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/strategies` | List all strategies |
| GET | `/api/v1/strategies/:id` | Get strategy details |
| POST | `/api/v1/strategies` | Create strategy |
| PUT | `/api/v1/strategies/:id` | Update strategy |
| DELETE | `/api/v1/strategies/:id` | Delete strategy |
| POST | `/api/v1/strategies/:id/start` | Start strategy |
| POST | `/api/v1/strategies/:id/stop` | Stop strategy |

### Pools

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/pools` | List available pools |
| GET | `/api/v1/pools/:address` | Get pool details |
| GET | `/api/v1/pools/:address/state` | Get current pool state |

### Analytics

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/analytics/portfolio` | Portfolio analytics |
| POST | `/api/v1/analytics/simulate` | Run simulation |

---

## 🤝 Contributing

We welcome contributions! Please follow these steps:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and ensure tests pass: `make pre-push`
4. Commit with conventional commits: `git commit -m 'feat: add amazing feature'`
5. Push to your fork: `git push origin feature/amazing-feature`
6. Open a Pull Request

### Development Guidelines

- Follow Rust idioms and best practices
- Add documentation for all public items
- Write tests for new functionality
- Keep PRs focused and atomic
- Use `make lint-fix` before committing

---

## 📞 Contact

- **Author**: Joaquín Béjar García
- **Email**: jb@taunais.com
- **Telegram**: [@joaquin_bejar](https://t.me/joaquin_bejar)
- **Repository**: <https://github.com/joaquinbejar/CLMM-Liquidity-Provider>
- **Documentation**: <https://docs.rs/clmm-liquidity-provider>

---

## ✍️ License

This project is dual-licensed under:

- **MIT License** ([LICENSE-MIT](./LICENSE-MIT) or http://opensource.org/licenses/MIT)
- **Apache License 2.0** ([LICENSE-APACHE](./LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

You may choose either license.

---

## ⚠️ Disclaimer

This software is provided for educational and research purposes. Trading cryptocurrencies and providing liquidity involves significant risk. The authors are not responsible for any financial losses incurred through the use of this software. Always do your own research and never invest more than you can afford to lose.