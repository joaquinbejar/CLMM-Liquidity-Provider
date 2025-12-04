use crate::token::TokenAmount;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

/// Calculates total fees earned by the pool given volume and fee tier.
pub fn calculate_pool_fees(volume: TokenAmount, fee_bps: u32) -> Result<TokenAmount, &'static str> {
    let vol = Decimal::from_str(&volume.0.to_string()).map_err(|_| "Conversion error")?;
    let bps = Decimal::from(fee_bps);
    let ten_thousand = Decimal::from(10000);

    let fees = vol * (bps / ten_thousand);

    // Convert back to TokenAmount (U256)
    // This is a bit rough, truncating decimals
    let fees_u128 = fees.to_u128().ok_or("Overflow")?;
    Ok(TokenAmount::from(fees_u128))
}

/// Calculates APY based on fees earned over a period
pub fn calculate_apy(
    fees_earned: Decimal,
    principal: Decimal,
    days: u32,
) -> Result<Decimal, &'static str> {
    if principal.is_zero() {
        return Err("Principal cannot be zero");
    }
    if days == 0 {
        return Err("Days cannot be zero");
    }

    let days_dec = Decimal::from(days);
    let year_days = Decimal::from(365);

    let roi = fees_earned / principal;
    let annualized = roi * (year_days / days_dec);

    Ok(annualized)
}
