use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Token {
    pub mint_address: String,
    pub symbol: String,
    pub decimals: u8,
    pub name: String,
    pub coingecko_id: Option<String>,
}

impl Token {
    pub fn new(
        mint: impl Into<String>,
        symbol: impl Into<String>,
        decimals: u8,
        name: impl Into<String>,
    ) -> Self {
        Self {
            mint_address: mint.into(),
            symbol: symbol.into(),
            decimals,
            name: name.into(),
            coingecko_id: None,
        }
    }
}
