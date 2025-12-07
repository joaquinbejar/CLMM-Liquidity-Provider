-- Initial database schema for CLMM LP Strategy Optimizer
-- Migration: 001_initial_schema

-- Pools table: stores CLMM pool configurations
CREATE TABLE IF NOT EXISTS pools (
    id UUID PRIMARY KEY,
    protocol VARCHAR(50) NOT NULL,  -- 'raydium', 'orca', 'meteora'
    address VARCHAR(64) NOT NULL UNIQUE,
    token_mint_a VARCHAR(64) NOT NULL,
    token_mint_b VARCHAR(64) NOT NULL,
    symbol_a VARCHAR(20) NOT NULL,
    symbol_b VARCHAR(20) NOT NULL,
    decimals_a SMALLINT NOT NULL,
    decimals_b SMALLINT NOT NULL,
    fee_tier INTEGER NOT NULL,  -- Fee in basis points (e.g., 30 = 0.3%)
    tick_spacing INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for protocol queries
CREATE INDEX IF NOT EXISTS idx_pools_protocol ON pools(protocol);
CREATE INDEX IF NOT EXISTS idx_pools_tokens ON pools(token_mint_a, token_mint_b);

-- Simulations table: stores backtest/simulation configurations
CREATE TABLE IF NOT EXISTS simulations (
    id UUID PRIMARY KEY,
    pool_id UUID REFERENCES pools(id) ON DELETE CASCADE,
    strategy_type VARCHAR(50) NOT NULL,  -- 'static', 'periodic', 'threshold'
    strategy_config JSONB NOT NULL DEFAULT '{}',
    start_timestamp BIGINT NOT NULL,  -- Unix timestamp in seconds
    end_timestamp BIGINT NOT NULL,
    initial_capital DECIMAL(20, 8) NOT NULL,
    entry_price DECIMAL(20, 8) NOT NULL,
    lower_price DECIMAL(20, 8) NOT NULL,
    upper_price DECIMAL(20, 8) NOT NULL,
    fee_rate DECIMAL(10, 6) NOT NULL,
    tx_cost DECIMAL(20, 8) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for pool queries
CREATE INDEX IF NOT EXISTS idx_simulations_pool ON simulations(pool_id);
CREATE INDEX IF NOT EXISTS idx_simulations_created ON simulations(created_at DESC);

-- Simulation results table: stores computed metrics for each simulation
CREATE TABLE IF NOT EXISTS simulation_results (
    id UUID PRIMARY KEY,
    simulation_id UUID NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    final_value DECIMAL(20, 8) NOT NULL,
    final_pnl DECIMAL(20, 8) NOT NULL,
    total_fees DECIMAL(20, 8) NOT NULL,
    total_il DECIMAL(20, 8) NOT NULL,
    final_il_pct DECIMAL(10, 6) NOT NULL,
    time_in_range_pct DECIMAL(10, 6) NOT NULL,
    max_drawdown DECIMAL(10, 6) NOT NULL,
    rebalance_count INTEGER NOT NULL DEFAULT 0,
    total_rebalance_cost DECIMAL(20, 8) NOT NULL DEFAULT 0,
    hodl_value DECIMAL(20, 8) NOT NULL,
    vs_hodl DECIMAL(20, 8) NOT NULL,
    sharpe_ratio DECIMAL(10, 6),
    final_price DECIMAL(20, 8) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for simulation queries
CREATE INDEX IF NOT EXISTS idx_results_simulation ON simulation_results(simulation_id);

-- Price history table: caches historical price data
CREATE TABLE IF NOT EXISTS price_history (
    id UUID PRIMARY KEY,
    pool_id UUID REFERENCES pools(id) ON DELETE CASCADE,
    timestamp BIGINT NOT NULL,  -- Unix timestamp in seconds
    open_price DECIMAL(20, 8) NOT NULL,
    high_price DECIMAL(20, 8) NOT NULL,
    low_price DECIMAL(20, 8) NOT NULL,
    close_price DECIMAL(20, 8) NOT NULL,
    volume DECIMAL(30, 8),
    liquidity DECIMAL(30, 8),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(pool_id, timestamp)
);

-- Index for time-series queries
CREATE INDEX IF NOT EXISTS idx_price_history_pool_time ON price_history(pool_id, timestamp DESC);

-- Optimization results table: stores optimization run results
CREATE TABLE IF NOT EXISTS optimization_results (
    id UUID PRIMARY KEY,
    pool_id UUID REFERENCES pools(id) ON DELETE CASCADE,
    objective_type VARCHAR(50) NOT NULL,  -- 'pnl', 'fees', 'sharpe'
    start_timestamp BIGINT NOT NULL,
    end_timestamp BIGINT NOT NULL,
    initial_capital DECIMAL(20, 8) NOT NULL,
    volatility DECIMAL(10, 6) NOT NULL,
    recommended_lower DECIMAL(20, 8) NOT NULL,
    recommended_upper DECIMAL(20, 8) NOT NULL,
    expected_pnl DECIMAL(20, 8) NOT NULL,
    expected_fees DECIMAL(20, 8) NOT NULL,
    expected_il DECIMAL(20, 8) NOT NULL,
    sharpe_ratio DECIMAL(10, 6),
    simulations_run INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for pool queries
CREATE INDEX IF NOT EXISTS idx_optimization_pool ON optimization_results(pool_id);
CREATE INDEX IF NOT EXISTS idx_optimization_created ON optimization_results(created_at DESC);
