use primitive_types::U256;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Token {
    pub address: String,
    pub symbol: String,
    pub decimals: u8,
    pub name: String,
}

impl Token {
    pub fn new(
        address: impl Into<String>,
        symbol: impl Into<String>,
        decimals: u8,
        name: impl Into<String>,
    ) -> Self {
        Self {
            address: address.into(),
            symbol: symbol.into(),
            decimals,
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TokenAmount(pub U256);

impl TokenAmount {
    pub fn new(amount: impl Into<U256>) -> Self {
        Self(amount.into())
    }

    pub fn zero() -> Self {
        Self(U256::zero())
    }

    pub fn as_u256(&self) -> U256 {
        self.0
    }
}

impl From<u64> for TokenAmount {
    fn from(v: u64) -> Self {
        Self(U256::from(v))
    }
}

impl From<u128> for TokenAmount {
    fn from(v: u128) -> Self {
        Self(U256::from(v))
    }
}

impl fmt::Display for TokenAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Price(pub Decimal);

impl Price {
    pub fn new(price: Decimal) -> Self {
        Self(price)
    }
}

impl From<Decimal> for Price {
    fn from(d: Decimal) -> Self {
        Self(d)
    }
}
