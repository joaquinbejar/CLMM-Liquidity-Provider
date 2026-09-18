# CLMM Liquidity Provider - Startup Guide

This guide explains how to start all services in the correct order to have the complete solution running.

## Prerequisites

Before starting, ensure you have the following installed:

- **Rust**: 1.75+ (`rustup update stable`)
- **Node.js**: 18+ (`node --version`)
- **PostgreSQL**: 14+ (running on port 5432)
- **Make**: Build automation tool

## Quick Start (TL;DR)

```bash
# 1. Setup environment
cp .env.example .env
# Edit .env with your values

# 2. Start PostgreSQL (if not running)
docker run -d --name clmm-postgres \
  -e POSTGRES_USER=clmm_user \
  -e POSTGRES_PASSWORD=clmm_password \
  -e POSTGRES_DB=clmm_lp \
  -p 5432:5432 postgres:14

# 3. Initialize database
cargo run --bin clmm-lp-cli -- db init

# 4. Start API server (Terminal 1)
cargo run --bin clmm-lp-api

# 5. Start Web Dashboard (Terminal 2)
cd web && npm install && npm run dev
```

---

## Detailed Startup Instructions

### Step 1: Environment Configuration

Copy the example environment file and configure it:

```bash
cp .env.example .env
```

Edit `.env` with your values. At minimum, configure:

```bash
# Required for data fetching
BIRDEYE_API_KEY=your_birdeye_api_key

# Required for database
DATABASE_URL=postgres://clmm_user:clmm_password@localhost:5432/clmm_lp

# Optional: Solana RPC (defaults to mainnet)
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
```

### Step 2: Start PostgreSQL Database

**Option A: Using Docker (Recommended)**

```bash
docker run -d \
  --name clmm-postgres \
  -e POSTGRES_USER=clmm_user \
  -e POSTGRES_PASSWORD=clmm_password \
  -e POSTGRES_DB=clmm_lp \
  -p 5432:5432 \
  postgres:14
```

**Option B: Using local PostgreSQL**

```bash
# Create database and user
psql -U postgres -c "CREATE USER clmm_user WITH PASSWORD 'clmm_password';"
psql -U postgres -c "CREATE DATABASE clmm_lp OWNER clmm_user;"
```

Verify connection:

```bash
psql postgres://clmm_user:clmm_password@localhost:5432/clmm_lp -c "SELECT 1;"
```

### Step 3: Build the Project

```bash
# Build all crates
make build

# Or with release optimizations
cargo build --release --workspace
```

### Step 4: Initialize Database Schema

Run migrations to create the required tables:

```bash
cargo run --bin clmm-lp-cli -- db init
```

Verify the database status:

```bash
cargo run --bin clmm-lp-cli -- db status
```

Expected output:
```
✅ Database connection successful
📊 Tables: pools, simulations, simulation_results, price_history, optimization_results
```

### Step 5: Start the API Server

Open a new terminal and start the API server:

```bash
# Development mode
cargo run --bin clmm-lp-api

# Or production mode
RUST_LOG=info cargo run --release --bin clmm-lp-api
```

The API server will start on `http://localhost:8080`.

Verify it's running:

```bash
curl http://localhost:8080/api/v1/health
```

Expected response:
```json
{"status":"healthy","version":"0.2.0"}
```

**Available endpoints:**
- REST API: `http://localhost:8080/api/v1`
- Swagger UI: `http://localhost:8080/docs`
- WebSocket: `ws://localhost:8080/ws`

### Step 6: Start the Web Dashboard

Open another terminal and start the web dashboard:

```bash
cd web

# Install dependencies (first time only)
npm install

# Start development server
npm run dev
```

The dashboard will be available at `http://localhost:3000`.

> **Note**: The dashboard requires the API server to be running on port 8080.

---

## Service Overview

| Service | Port | URL | Description |
|---------|------|-----|-------------|
| PostgreSQL | 5432 | `localhost:5432` | Database |
| API Server | 8080 | `http://localhost:8080` | REST API + WebSocket |
| Swagger UI | 8080 | `http://localhost:8080/docs` | API Documentation |
| Web Dashboard | 3000 | `http://localhost:3000` | React Frontend |

---

## Startup Order

The services must be started in this order:

```
1. PostgreSQL Database
       ↓
2. Database Initialization (one-time)
       ↓
3. API Server
       ↓
4. Web Dashboard
```

---

## Using the CLI

The CLI can be used independently without the API server:

```bash
# Analyze a trading pair
cargo run --bin clmm-lp-cli -- analyze \
  --symbol-a SOL \
  --symbol-b USDC \
  --days 30

# Run a backtest
cargo run --bin clmm-lp-cli -- backtest \
  --symbol-a SOL \
  --symbol-b USDC \
  --capital 10000 \
  --lower-price 80 \
  --upper-price 120 \
  --strategy periodic

# Optimize range parameters
cargo run --bin clmm-lp-cli -- optimize \
  --symbol-a SOL \
  --symbol-b USDC \
  --capital 10000 \
  --objective sharpe
```

---

## Docker Compose (Full Stack)

For convenience, you can use Docker Compose to start all services:

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: postgres:18
    environment:
      POSTGRES_USER: clmm_user
      POSTGRES_PASSWORD: clmm_password
      POSTGRES_DB: clmm_lp
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  api:
    build: .
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://clmm_user:clmm_password@postgres:5432/clmm_lp
      RUST_LOG: info
    depends_on:
      - postgres

  web:
    build: ./web
    ports:
      - "3000:3000"
    depends_on:
      - api

volumes:
  postgres_data:
```

Start with:

```bash
docker-compose up -d
```

---

## Troubleshooting

### Database Connection Failed

```
Error: password authentication failed for user "joaquin"
```

**Solution**: Ensure `DATABASE_URL` is set in your `.env` file and you're running from the project root:

```bash
cd /path/to/CLMM-Liquidity-Provider
cargo run --bin clmm-lp-cli -- db init
```

### API Server Port Already in Use

```
Error: Address already in use (os error 48)
```

**Solution**: Change the port or kill the existing process:

```bash
# Find process using port 8080
lsof -i :8080

# Kill it
kill -9 <PID>

# Or use a different port
API_PORT=8081 cargo run --bin clmm-lp-api
```

### Web Dashboard Proxy Errors

```
[vite] http proxy error: /api/v1/health
AggregateError [ECONNREFUSED]
```

**Solution**: The API server is not running. Start it first:

```bash
cargo run --bin clmm-lp-api
```

### Missing Birdeye API Key

```
Error: BIRDEYE_API_KEY not set
```

**Solution**: Add your Birdeye API key to `.env`:

```bash
BIRDEYE_API_KEY=your_api_key_here
```

Get an API key at: https://birdeye.so/

---

## Health Checks

Verify all services are running:

```bash
# Check PostgreSQL
pg_isready -h localhost -p 5432

# Check API Server
curl -s http://localhost:8080/api/v1/health | jq

# Check Web Dashboard
curl -s http://localhost:3000 | head -1
```

---

## Stopping Services

```bash
# Stop Web Dashboard
# Press Ctrl+C in the terminal running npm

# Stop API Server
# Press Ctrl+C in the terminal running cargo

# Stop PostgreSQL (Docker)
docker stop clmm-postgres

# Stop all (Docker Compose)
docker-compose down
```

---

## Next Steps

Once all services are running:

1. Open the **Web Dashboard** at `http://localhost:3000`
2. Explore the **Swagger UI** at `http://localhost:8080/docs`
3. Run your first **analysis** with the CLI
4. Configure a **strategy** and start monitoring positions

For more information, see the [README.md](./README.md).
