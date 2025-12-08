-- Migration: 002_add_positions
-- Adds position tracking tables for live position management

-- Positions table: tracks live LP positions
CREATE TABLE IF NOT EXISTS positions (
    id UUID PRIMARY KEY,
    pool_id UUID REFERENCES pools(id) ON DELETE CASCADE,
    owner_address VARCHAR(64) NOT NULL,
    position_address VARCHAR(64) NOT NULL UNIQUE,
    tick_lower INTEGER NOT NULL,
    tick_upper INTEGER NOT NULL,
    liquidity DECIMAL(40, 0) NOT NULL,
    entry_price DECIMAL(20, 8) NOT NULL,
    entry_timestamp BIGINT NOT NULL,
    token_a_deposited DECIMAL(20, 8) NOT NULL,
    token_b_deposited DECIMAL(20, 8) NOT NULL,
    fees_a_collected DECIMAL(20, 8) NOT NULL DEFAULT 0,
    fees_b_collected DECIMAL(20, 8) NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'active',  -- 'active', 'closed', 'pending'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMPTZ
);

-- Indexes for position queries
CREATE INDEX IF NOT EXISTS idx_positions_pool ON positions(pool_id);
CREATE INDEX IF NOT EXISTS idx_positions_owner ON positions(owner_address);
CREATE INDEX IF NOT EXISTS idx_positions_status ON positions(status);
CREATE INDEX IF NOT EXISTS idx_positions_address ON positions(position_address);

-- Position events table: tracks position lifecycle events
CREATE TABLE IF NOT EXISTS position_events (
    id UUID PRIMARY KEY,
    position_id UUID NOT NULL REFERENCES positions(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,  -- 'opened', 'rebalanced', 'fees_collected', 'closed'
    event_data JSONB NOT NULL DEFAULT '{}',
    tx_signature VARCHAR(128),
    tx_cost_lamports BIGINT NOT NULL DEFAULT 0,
    timestamp BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for event queries
CREATE INDEX IF NOT EXISTS idx_events_position ON position_events(position_id);
CREATE INDEX IF NOT EXISTS idx_events_type ON position_events(event_type);
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON position_events(timestamp DESC);

-- Strategies table: stores strategy configurations
CREATE TABLE IF NOT EXISTS strategies (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    strategy_type VARCHAR(50) NOT NULL,  -- 'static', 'periodic', 'threshold', 'il_limit'
    config JSONB NOT NULL DEFAULT '{}',
    pool_id UUID REFERENCES pools(id) ON DELETE SET NULL,
    is_active BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for strategy queries
CREATE INDEX IF NOT EXISTS idx_strategies_pool ON strategies(pool_id);
CREATE INDEX IF NOT EXISTS idx_strategies_active ON strategies(is_active);

-- Strategy executions table: tracks strategy execution history
CREATE TABLE IF NOT EXISTS strategy_executions (
    id UUID PRIMARY KEY,
    strategy_id UUID NOT NULL REFERENCES strategies(id) ON DELETE CASCADE,
    position_id UUID REFERENCES positions(id) ON DELETE SET NULL,
    action VARCHAR(50) NOT NULL,  -- 'evaluate', 'rebalance', 'close', 'open'
    decision JSONB NOT NULL DEFAULT '{}',
    executed BOOLEAN NOT NULL DEFAULT false,
    error_message TEXT,
    timestamp BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for execution queries
CREATE INDEX IF NOT EXISTS idx_executions_strategy ON strategy_executions(strategy_id);
CREATE INDEX IF NOT EXISTS idx_executions_timestamp ON strategy_executions(timestamp DESC);

-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert migration record
INSERT INTO schema_migrations (version, name) 
VALUES (2, '002_add_positions')
ON CONFLICT (version) DO NOTHING;
