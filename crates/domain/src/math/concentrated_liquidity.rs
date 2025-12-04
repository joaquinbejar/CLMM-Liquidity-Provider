use crate::token::TokenAmount;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

/// Calculates the amount of token0 (x) given liquidity and price range.
/// delta_x = L * (1/sqrt(P_a) - 1/sqrt(P_b))
/// where P_a < P_b
pub fn get_amount0_delta(
    liquidity: u128,
    sqrt_price_a: Decimal,
    sqrt_price_b: Decimal,
) -> Result<TokenAmount, &'static str> {
    if sqrt_price_a <= Decimal::ZERO || sqrt_price_b <= Decimal::ZERO {
        return Err("Sqrt price must be positive");
    }

    let (lower, upper) = if sqrt_price_a < sqrt_price_b {
        (sqrt_price_a, sqrt_price_b)
    } else {
        (sqrt_price_b, sqrt_price_a)
    };

    // delta_x = L * ( (upper - lower) / (lower * upper) )
    // using rust_decimal for precision

    let liquidity_dec = Decimal::from(liquidity);

    let num = upper - lower;
    let den = lower * upper;

    if den.is_zero() {
        return Err("Denominator zero");
    }

    let factor = num / den;
    let amount = liquidity_dec * factor;

    let amount_u128 = amount.to_u128().ok_or("Overflow converting amount")?;
    Ok(TokenAmount::from(amount_u128))
}

/// Calculates the amount of token1 (y) given liquidity and price range.
/// delta_y = L * (sqrt(P_b) - sqrt(P_a))
/// where P_a < P_b
pub fn get_amount1_delta(
    liquidity: u128,
    sqrt_price_a: Decimal,
    sqrt_price_b: Decimal,
) -> Result<TokenAmount, &'static str> {
    let (lower, upper) = if sqrt_price_a < sqrt_price_b {
        (sqrt_price_a, sqrt_price_b)
    } else {
        (sqrt_price_b, sqrt_price_a)
    };

    let liquidity_dec = Decimal::from(liquidity);
    let diff = upper - lower;

    let amount = liquidity_dec * diff;

    let amount_u128 = amount.to_u128().ok_or("Overflow converting amount")?;
    Ok(TokenAmount::from(amount_u128))
}

/// Calculates liquidity for a given amount of token0 and price range
/// L = amount0 * (sqrt(P_a) * sqrt(P_b)) / (sqrt(P_b) - sqrt(P_a))
pub fn get_liquidity_for_amount0(
    amount0: TokenAmount,
    sqrt_price_a: Decimal,
    sqrt_price_b: Decimal,
) -> Result<u128, &'static str> {
    let (lower, upper) = if sqrt_price_a < sqrt_price_b {
        (sqrt_price_a, sqrt_price_b)
    } else {
        (sqrt_price_b, sqrt_price_a)
    };

    let amount0_dec = Decimal::from_str(&amount0.0.to_string()).map_err(|_| "Conversion error")?;

    let num = amount0_dec * lower * upper;
    let den = upper - lower;

    if den.is_zero() {
        return Err("Range too small");
    }

    let liquidity = num / den;
    liquidity.to_u128().ok_or("Overflow")
}

/// Calculates liquidity for a given amount of token1 and price range
/// L = amount1 / (sqrt(P_b) - sqrt(P_a))
pub fn get_liquidity_for_amount1(
    amount1: TokenAmount,
    sqrt_price_a: Decimal,
    sqrt_price_b: Decimal,
) -> Result<u128, &'static str> {
    let (lower, upper) = if sqrt_price_a < sqrt_price_b {
        (sqrt_price_a, sqrt_price_b)
    } else {
        (sqrt_price_b, sqrt_price_a)
    };

    let amount1_dec = Decimal::from_str(&amount1.0.to_string()).map_err(|_| "Conversion error")?;

    let den = upper - lower;
    if den.is_zero() {
        return Err("Range too small");
    }

    let liquidity = amount1_dec / den;
    liquidity.to_u128().ok_or("Overflow")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_deltas() {
        // Example: Liquidity 1000
        // Price goes from 1 to 4 (sqrt: 1 to 2)
        // delta_y = 1000 * (2 - 1) = 1000
        // delta_x = 1000 * (1/1 - 1/2) = 1000 * 0.5 = 500

        let liquidity = 1000u128;
        let sqrt_p_a = Decimal::from(1);
        let sqrt_p_b = Decimal::from(2);

        let dy = get_amount1_delta(liquidity, sqrt_p_a, sqrt_p_b).unwrap();
        assert_eq!(dy.as_u256().as_u64(), 1000);

        let dx = get_amount0_delta(liquidity, sqrt_p_a, sqrt_p_b).unwrap();
        assert_eq!(dx.as_u256().as_u64(), 500);
    }

    #[test]
    fn test_get_liquidity() {
        let sqrt_p_a = Decimal::from(1);
        let sqrt_p_b = Decimal::from(2);

        // From previous test: if dx = 500, L should be 1000
        let dx = TokenAmount::from(500u64);
        let l = get_liquidity_for_amount0(dx, sqrt_p_a, sqrt_p_b).unwrap();
        assert_eq!(l, 1000);

        // If dy = 1000, L should be 1000
        let dy = TokenAmount::from(1000u64);
        let l2 = get_liquidity_for_amount1(dy, sqrt_p_a, sqrt_p_b).unwrap();
        assert_eq!(l2, 1000);
    }
}
