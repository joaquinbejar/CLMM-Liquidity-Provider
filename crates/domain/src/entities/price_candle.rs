use crate::entities::token::Token;
use crate::value_objects::amount::Amount;
use crate::value_objects::price::Price;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCandle {
    pub token_a: Token,
    pub token_b: Token,
    pub start_timestamp: u64,
    pub duration_seconds: u64,
    
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    
    pub volume_token_a: Amount,
}
