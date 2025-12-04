use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use primitive_types::U256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Amount {
    pub raw: U256,
    pub decimals: u8,
}

impl Amount {
    pub fn new(raw: U256, decimals: u8) -> Self {
        Self { raw, decimals }
    }

    pub fn from_decimal(d: Decimal, decimals: u8) -> Self {
        let multiplier = Decimal::from(10u64.pow(decimals as u32));
        let raw_decimal = d * multiplier;
        let raw_u128 = raw_decimal.to_u128().unwrap_or(0); // Handle overflow gracefully or better
        Self {
            raw: U256::from(raw_u128),
            decimals,
        }
    }

    pub fn to_decimal(&self) -> Decimal {
        let raw_u128 = self.raw.as_u128(); // Warning: U256 to u128 might truncate if huge
        let d = Decimal::from(raw_u128);
        let divisor = Decimal::from(10u64.pow(self.decimals as u32));
        d / divisor
    }
}
