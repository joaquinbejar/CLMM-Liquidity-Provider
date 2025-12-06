use rust_decimal::Decimal;

/// Trait to model the global liquidity of a pool.
pub trait LiquidityModel {
    /// Returns the global active liquidity at a given price.
    fn get_liquidity_at_price(&self, price: Decimal) -> u128;
}

/// A simple model with constant global liquidity.
#[derive(Debug, Clone)]
pub struct ConstantLiquidity {
    /// The constant liquidity value.
    pub liquidity: u128,
}

impl ConstantLiquidity {
    /// Creates a new ConstantLiquidity model.
    pub fn new(liquidity: u128) -> Self {
        Self { liquidity }
    }
}

impl LiquidityModel for ConstantLiquidity {
    fn get_liquidity_at_price(&self, _price: Decimal) -> u128 {
        self.liquidity
    }
}
