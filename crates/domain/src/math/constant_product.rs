use crate::token::TokenAmount;
use primitive_types::U256;

/// Calculates the output amount for a given input amount in a constant product pool (x * y = k).
/// returns (output_amount, new_reserve_in, new_reserve_out)
///
/// formula: dy = y * dx / (x + dx)
/// taking fee into account: dy = y * (dx * (1 - fee)) / (x + (dx * (1 - fee)))
pub fn calculate_out_amount(
    amount_in: TokenAmount,
    reserve_in: TokenAmount,
    reserve_out: TokenAmount,
    fee_bps: u32,
) -> Result<TokenAmount, &'static str> {
    let amount_in = amount_in.0;
    let reserve_in = reserve_in.0;
    let reserve_out = reserve_out.0;

    if amount_in.is_zero() {
        return Ok(TokenAmount::zero());
    }
    if reserve_in.is_zero() || reserve_out.is_zero() {
        return Err("Reserves must be non-zero");
    }

    let amount_in_with_fee = amount_in
        .checked_mul(U256::from(10000 - fee_bps))
        .ok_or("Overflow")?;
    let numerator = amount_in_with_fee
        .checked_mul(reserve_out)
        .ok_or("Overflow")?;
    let denominator = reserve_in
        .checked_mul(U256::from(10000))
        .ok_or("Overflow")?
        .checked_add(amount_in_with_fee)
        .ok_or("Overflow")?;

    let amount_out = numerator / denominator;

    Ok(TokenAmount(amount_out))
}

/// Calculates the spot price of token_in in terms of token_out
/// Price = reserve_out / reserve_in
pub fn calculate_spot_price(
    reserve_in: TokenAmount,
    reserve_out: TokenAmount,
) -> Result<rust_decimal::Decimal, &'static str> {
    use rust_decimal::prelude::*;

    // We need to be careful with precision here. U256 to Decimal conversion might need handling.
    // For now, assuming simple conversion works for reasonable reserve sizes.
    // A better approach is to use big decimal or string parsing.

    let r_in = Decimal::from_str(&reserve_in.0.to_string()).map_err(|_| "Conversion error")?;
    let r_out = Decimal::from_str(&reserve_out.0.to_string()).map_err(|_| "Conversion error")?;

    if r_in.is_zero() {
        return Err("Reserve in is zero");
    }

    Ok(r_out / r_in)
}

/// Calculates the constant product K
pub fn calculate_k(reserve0: TokenAmount, reserve1: TokenAmount) -> U256 {
    reserve0.0.saturating_mul(reserve1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_calculate_out_amount() {
        // 1000 reserve0, 1000 reserve1, 10 input, 0.3% fee (30 bps)
        // amount_in_with_fee = 10 * 9970 = 99700 (scaled by 10000)
        // numerator = 99700 * 1000 = 99,700,000
        // denominator = 1000 * 10000 + 99700 = 10,099,700
        // out = 99,700,000 / 10,099,700 = 9.8715... -> 9

        let r0 = TokenAmount::from(1000u64);
        let r1 = TokenAmount::from(1000u64);
        let input = TokenAmount::from(10u64);
        let fee = 30;

        let out = calculate_out_amount(input, r0, r1, fee).unwrap();
        assert_eq!(out.0.as_u64(), 9);
    }

    #[test]
    fn test_calculate_spot_price() {
        let r0 = TokenAmount::from(2000u64);
        let r1 = TokenAmount::from(1000u64);

        let price = calculate_spot_price(r0, r1).unwrap();
        // price = 1000 / 2000 = 0.5
        assert_eq!(price, rust_decimal::Decimal::from_str("0.5").unwrap());
    }
}
