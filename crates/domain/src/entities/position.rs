use crate::enums::PositionStatus;
use crate::value_objects::{amount::Amount, price_range::PriceRange};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PositionId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: PositionId,
    pub pool_address: String,
    pub owner_address: String,

    pub liquidity_amount: u128,

    pub deposited_amount_a: Amount,
    pub deposited_amount_b: Amount,

    pub current_amount_a: Amount,
    pub current_amount_b: Amount,

    pub unclaimed_fees_a: Amount,
    pub unclaimed_fees_b: Amount,

    pub range: Option<PriceRange>, // For CLMM

    pub opened_at: u64,
    pub status: PositionStatus,
}
