use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Percentage(pub Decimal);

impl Percentage {
    pub fn from_bps(bps: u32) -> Self {
        Self(Decimal::from(bps) / Decimal::from(10000))
    }

    pub fn to_bps(&self) -> u32 {
        (self.0 * Decimal::from(10000)).to_u32().unwrap_or(0)
    }
}
