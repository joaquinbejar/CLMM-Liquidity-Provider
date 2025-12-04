use crate::token::TokenAmount;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FeeTier {
    pub bps: u32,
    pub tick_spacing: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeAccumulation {
    pub amount0: TokenAmount,
    pub amount1: TokenAmount,
    pub uncollected0: TokenAmount,
    pub uncollected1: TokenAmount,
}
