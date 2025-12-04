use crate::token::{Price, Token, TokenAmount};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoolType {
    ConstantProduct,
    ConcentratedLiquidity,
    StableSwap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    pub id: String,
    pub address: String,
    pub chain_id: u64,
    pub token0: Token,
    pub token1: Token,
    pub fee_tier: u32, // in bps, e.g. 3000 for 0.3%
    pub pool_type: PoolType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolState {
    pub pool_id: String,
    pub reserve0: TokenAmount,
    pub reserve1: TokenAmount,
    pub price: Price,
    pub tick: Option<i32>,       // for concentrated liquidity
    pub liquidity: Option<u128>, // for concentrated liquidity
    pub timestamp: u64,
}
