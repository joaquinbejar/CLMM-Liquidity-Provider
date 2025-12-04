use rust_decimal::Decimal;
use rust_decimal::prelude::*;

/// Returns the price corresponding to a given tick.
/// P = 1.0001 ^ tick
pub fn tick_to_price(tick: i32) -> Result<Decimal, &'static str> {
    let base = 1.0001f64;
    let price_f64 = base.powi(tick);
    Decimal::from_f64(price_f64).ok_or("Overflow converting price")
}

/// Returns the tick corresponding to a given price.
/// tick = log_1.0001(P)
pub fn price_to_tick(price: Decimal) -> Result<i32, &'static str> {
    if price <= Decimal::ZERO {
        return Err("Price must be positive");
    }
    let price_f64 = price.to_f64().ok_or("Overflow converting price")?;
    let base = 1.0001f64;
    let tick = price_f64.log(base);
    Ok(tick.round() as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_to_price() {
        // Tick 0 -> Price 1
        let p = tick_to_price(0).unwrap();
        assert_eq!(p, Decimal::from(1));

        // Tick 100 -> 1.0001^100 ~= 1.010049
        let p100 = tick_to_price(100).unwrap();
        // Allow small error due to f64
        let expected = 1.01004966;
        let diff = (p100.to_f64().unwrap() - expected).abs();
        assert!(diff < 0.000001);
    }

    #[test]
    fn test_price_to_tick() {
        let t = price_to_tick(Decimal::from(1)).unwrap();
        assert_eq!(t, 0);

        let t2 = price_to_tick(Decimal::from_f64(1.01004966).unwrap()).unwrap();
        assert_eq!(t2, 100);
    }
}
