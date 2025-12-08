// API client for CLMM LP backend

const API_BASE = '/api/v1'

export interface Position {
  address: string
  pool_address: string
  owner: string
  tick_lower: number
  tick_upper: number
  liquidity: string
  in_range: boolean
  value_usd: string
  pnl: PnL
  status: 'active' | 'closed' | 'pending'
  created_at: string | null
}

export interface PnL {
  unrealized_pnl_usd: string
  unrealized_pnl_pct: string
  fees_earned_a: number
  fees_earned_b: number
  fees_earned_usd: string
  il_pct: string
  net_pnl_usd: string
  net_pnl_pct: string
}

export interface Strategy {
  id: string
  name: string
  description: string | null
  strategy_type: 'static' | 'periodic' | 'threshold' | 'il_limit'
  pool_address: string | null
  running: boolean
  parameters: StrategyParameters
  created_at: string
  updated_at: string
}

export interface StrategyParameters {
  rebalance_threshold_pct?: number
  max_il_pct?: number
  min_rebalance_interval_hours?: number
  range_width_pct?: number
}

export interface Pool {
  address: string
  protocol: string
  token_a: TokenInfo
  token_b: TokenInfo
  fee_tier: number
  tick_spacing: number
  tvl_usd: string
  volume_24h_usd: string
  fee_apy: string
}

export interface TokenInfo {
  mint: string
  symbol: string
  decimals: number
}

export interface PoolState {
  address: string
  current_tick: number
  sqrt_price: string
  liquidity: string
  token_a_reserve: string
  token_b_reserve: string
  last_updated: string
}

export interface PortfolioAnalytics {
  total_value_usd: string
  total_pnl_usd: string
  total_pnl_pct: string
  total_fees_usd: string
  total_il_usd: string
  active_positions: number
  best_performer: string | null
  worst_performer: string | null
}

export interface SimulationRequest {
  pool_address: string
  strategy_type: string
  tick_lower: number
  tick_upper: number
  initial_capital: number
  start_date: string
  end_date: string
}

export interface SimulationResponse {
  final_value: string
  total_return_pct: string
  fees_earned: string
  impermanent_loss: string
  max_drawdown: string
  sharpe_ratio: string | null
  time_in_range_pct: string
  rebalance_count: number
}

export interface HealthResponse {
  status: string
  version: string
  uptime_secs: number
  components: Record<string, ComponentHealth>
}

export interface ComponentHealth {
  status: string
  latency_ms: number | null
  message: string | null
}

// API functions

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE}${url}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  })
  
  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'Unknown error' }))
    throw new Error(error.message || `HTTP ${response.status}`)
  }
  
  return response.json()
}

// Health
export const getHealth = () => fetchJson<HealthResponse>('/health')

// Positions
export const getPositions = () => fetchJson<{ positions: Position[] }>('/positions')
export const getPosition = (address: string) => fetchJson<Position>(`/positions/${address}`)
export const openPosition = (data: {
  pool_address: string
  tick_lower: number
  tick_upper: number
  amount_a: number
  amount_b: number
}) => fetchJson<{ message: string }>('/positions', { method: 'POST', body: JSON.stringify(data) })
export const closePosition = (address: string) => 
  fetchJson<{ message: string }>(`/positions/${address}`, { method: 'DELETE' })
export const collectFees = (address: string) => 
  fetchJson<{ message: string }>(`/positions/${address}/collect`, { method: 'POST' })
export const rebalancePosition = (address: string, data: { new_tick_lower: number; new_tick_upper: number }) =>
  fetchJson<{ message: string }>(`/positions/${address}/rebalance`, { method: 'POST', body: JSON.stringify(data) })

// Strategies
export const getStrategies = () => fetchJson<{ strategies: Strategy[] }>('/strategies')
export const getStrategy = (id: string) => fetchJson<Strategy>(`/strategies/${id}`)
export const createStrategy = (data: Partial<Strategy>) =>
  fetchJson<Strategy>('/strategies', { method: 'POST', body: JSON.stringify(data) })
export const updateStrategy = (id: string, data: Partial<Strategy>) =>
  fetchJson<Strategy>(`/strategies/${id}`, { method: 'PUT', body: JSON.stringify(data) })
export const deleteStrategy = (id: string) =>
  fetchJson<{ message: string }>(`/strategies/${id}`, { method: 'DELETE' })
export const startStrategy = (id: string) =>
  fetchJson<{ message: string }>(`/strategies/${id}/start`, { method: 'POST' })
export const stopStrategy = (id: string) =>
  fetchJson<{ message: string }>(`/strategies/${id}/stop`, { method: 'POST' })

// Pools
export const getPools = () => fetchJson<{ pools: Pool[] }>('/pools')
export const getPool = (address: string) => fetchJson<Pool>(`/pools/${address}`)
export const getPoolState = (address: string) => fetchJson<PoolState>(`/pools/${address}/state`)

// Analytics
export const getPortfolioAnalytics = () => fetchJson<PortfolioAnalytics>('/analytics/portfolio')
export const runSimulation = (data: SimulationRequest) =>
  fetchJson<SimulationResponse>('/analytics/simulate', { method: 'POST', body: JSON.stringify(data) })
