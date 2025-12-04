use crate::entities::token::Token;
use crate::value_objects::amount::Amount;
use crate::enums::{Protocol, PoolType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    pub address: String,
    pub protocol: Protocol,
    pub pool_type: PoolType,
    pub token_a: Token,
    pub token_b: Token,
    pub reserve_a: Amount,
    pub reserve_b: Amount,
    pub fee_rate: u32, // bps
    
    // Specific to CLMM
    pub tick_spacing: Option<i32>,
    pub current_tick: Option<i32>,
    pub liquidity: Option<u128>,
    
    // Specific to Stable
    pub amplification_coefficient: Option<u64>,
    
    pub created_at: u64,
}
