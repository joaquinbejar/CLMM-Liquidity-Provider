use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Price {
    pub value: Decimal,
}

impl Price {
    pub fn new(value: Decimal) -> Self {
        Self { value }
    }

    pub fn invert(&self) -> Self {
        if self.value.is_zero() {
            return Self {
                value: Decimal::ZERO,
            };
        }
        Self {
            value: Decimal::ONE / self.value,
        }
    }
}
