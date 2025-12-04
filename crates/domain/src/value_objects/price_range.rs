use crate::value_objects::price::Price;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRange {
    pub lower_price: Price,
    pub upper_price: Price,
}

impl PriceRange {
    pub fn new(lower: Price, upper: Price) -> Self {
        Self {
            lower_price: lower,
            upper_price: upper,
        }
    }

    pub fn contains(&self, price: Price) -> bool {
        price.value >= self.lower_price.value && price.value <= self.upper_price.value
    }
}
