use crate::math::concentrated_liquidity;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

/// Calculates Impermanent Loss for a constant product pool.
/// formula: 2 * sqrt(price_ratio) / (1 + price_ratio) - 1
///
/// # Arguments
///
/// * `entry_price` - The price at which the position was opened (token1/token0)
/// * `current_price` - The current price (token1/token0)
///
/// # Returns
///
/// * `Decimal` - The impermanent loss as a negative percentage (e.g., -0.05 for 5% loss)
pub fn calculate_il_constant_product(
    entry_price: Decimal,
    current_price: Decimal,
) -> Result<Decimal, &'static str> {
    if entry_price.is_zero() {
        return Err("Entry price cannot be zero");
    }

    let price_ratio = current_price / entry_price;

    // sqrt is not directly available on Decimal in all versions/features,
    // but rust_decimal typically supports it via `MathematicalOps` feature or similar if enabled.
    // Since we added `rust_decimal = "1.33"`, we should check if `maths` feature is needed.
    // For now, we can convert to f64 for sqrt and back, or assume feature is available.
    // Let's use f64 for simplicity as IL is an estimation.

    let ratio_f64 = price_ratio.to_f64().ok_or("Overflow converting to f64")?;
    let sqrt_ratio = ratio_f64.sqrt();

    let numerator = 2.0 * sqrt_ratio;
    let denominator = 1.0 + ratio_f64;

    let result_f64 = (numerator / denominator) - 1.0;

    Decimal::from_f64(result_f64).ok_or("Overflow converting result")
}

/// Calculates Impermanent Loss for a concentrated liquidity position.
/// This compares the value of the LP position at current_price vs holding the initial assets.
pub fn calculate_il_concentrated(
    entry_price: Decimal,
    current_price: Decimal,
    price_lower: Decimal,
    price_upper: Decimal,
) -> Result<Decimal, &'static str> {
    if entry_price.is_zero() || price_lower.is_zero() || price_upper.is_zero() {
        return Err("Prices must be non-zero");
    }
    if price_lower >= price_upper {
        return Err("Invalid range");
    }

    // Arbitrary liquidity to simulate amounts.
    // Using a large number to avoid small number precision issues with integer TokenAmount.
    let liquidity = 1_000_000_000_000_000_000u128; // 1e18

    let sqrt = |p: Decimal| -> Result<Decimal, &'static str> {
        let f = p.to_f64().ok_or("Overflow")?;
        Decimal::from_f64(f.sqrt()).ok_or("Overflow")
    };

    let sqrt_entry = sqrt(entry_price)?;
    let sqrt_curr = sqrt(current_price)?;
    let sqrt_lower = sqrt(price_lower)?;
    let sqrt_upper = sqrt(price_upper)?;

    // 1. Calculate Initial Amounts (Held)
    // We need to know "active price" at entry to determine amounts.
    // If entry < lower, all X. If entry > upper, all Y. If in range, mix.
    // However, for IL calculation, we assume the position was created *at* entry_price.
    // So we use the standard liquidity formulas with entry_price as the "current" price for initial state.

    // BUT wait: get_amount functions take a range.
    // For amount0: range is [max(current, lower), upper]? No.
    // Uniswap logic:
    // if P < lower: amounts are determined by range [lower, upper] assuming P is below. All X.
    // actually, standard delta formulas work if we pass the range correctly.

    // Let's use a helper to get amounts at a specific price P for range [Lower, Upper]
    let get_amounts = |p_sqrt: Decimal| -> Result<(Decimal, Decimal), &'static str> {
        let mut amt0 = Decimal::ZERO;
        let mut amt1 = Decimal::ZERO;

        // If P < Lower: Price is below range. Position is all Token0 (X).
        // Effectively P_current = Lower for the purpose of logic? No.
        // Standard logic:
        // Liquidity is active only in [Lower, Upper].
        // If P < Lower: The curve segment is "above" us. We hold amount0 required to cross [Lower, Upper].
        // i.e., we are full in X.

        if p_sqrt < sqrt_lower {
            // Full range crossing for X
            let a0 = concentrated_liquidity::get_amount0_delta(liquidity, sqrt_lower, sqrt_upper)?;
            amt0 = Decimal::from_str(&a0.0.to_string()).unwrap();
        } else if p_sqrt >= sqrt_upper {
            // Price > Upper. Position is all Token1 (Y).
            let a1 = concentrated_liquidity::get_amount1_delta(liquidity, sqrt_lower, sqrt_upper)?;
            amt1 = Decimal::from_str(&a1.0.to_string()).unwrap();
        } else {
            // In range.
            // X part: from P to Upper
            let a0 = concentrated_liquidity::get_amount0_delta(liquidity, p_sqrt, sqrt_upper)?;
            amt0 = Decimal::from_str(&a0.0.to_string()).unwrap();
            // Y part: from Lower to P
            let a1 = concentrated_liquidity::get_amount1_delta(liquidity, sqrt_lower, p_sqrt)?;
            amt1 = Decimal::from_str(&a1.0.to_string()).unwrap();
        }
        Ok((amt0, amt1))
    };

    let (x0, y0) = get_amounts(sqrt_entry)?;
    let (x1, y1) = get_amounts(sqrt_curr)?;

    // Value Held: The initial bundle (x0, y0) valued at current_price
    let value_held = x0 * current_price + y0;

    // Value LP: The current bundle (x1, y1) valued at current_price
    let value_lp = x1 * current_price + y1;

    if value_held.is_zero() {
        // If we held nothing, no loss/gain reference. (Should not happen with non-zero liq)
        return Ok(Decimal::ZERO);
    }

    let il = (value_lp - value_held) / value_held;
    Ok(il)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_il_constant_product() {
        // Price doubles: 100 -> 200. Ratio = 2.
        // IL = 2*sqrt(2)/(1+2) - 1 = 2*1.4142/3 - 1 = 0.9428 - 1 = -0.0572 (5.72%)
        let entry = Decimal::from(100);
        let curr = Decimal::from(200);
        let il = calculate_il_constant_product(entry, curr).unwrap();

        let expected = Decimal::from_f64(-0.05719).unwrap();
        let diff = (il - expected).abs();
        assert!(diff < Decimal::from_f64(0.0001).unwrap());
    }

    #[test]
    fn test_calculate_il_concentrated() {
        // Range 90-110. Entry 100. Current 100. IL should be 0.
        let entry = Decimal::from(100);
        let curr = Decimal::from(100);
        let lower = Decimal::from(90);
        let upper = Decimal::from(110);

        let il = calculate_il_concentrated(entry, curr, lower, upper).unwrap();
        assert!(il.abs() < Decimal::from_f64(0.000001).unwrap());

        // Price moves to 105. Held: (x0, y0) at 105. LP: (x1, y1) at 105.
        // Since 105 is in range, we sold some X for Y (or vice versa) compared to held?
        // Actually, as price goes up, we sell X for Y.
        // Held (Hodl) would keep X. LP sold X.
        // Since Price went UP, Y is worth "less" relative to X? No, Price is Y/X.
        // Price Up -> X is more valuable in terms of Y? No.
        // Price = Y / X. (How much Y for 1 X).
        // If Price Up -> 1 X is worth MORE Y.
        // LP sells X as price goes up. So LP holds LESS of the appreciating asset (X).
        // Thus LP Value < Held Value. IL should be negative.

        let curr_up = Decimal::from(105);
        let il_up = calculate_il_concentrated(entry, curr_up, lower, upper).unwrap();
        assert!(il_up < Decimal::ZERO);
    }
}
