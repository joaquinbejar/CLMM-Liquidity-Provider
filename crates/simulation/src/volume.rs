use clmm_lp_domain::value_objects::amount::Amount;
use rust_decimal::Decimal;

/// Trait for modeling volume.
pub trait VolumeModel {
    /// Returns the volume for the next step as Amount.
    fn next_volume(&mut self) -> Amount;

    /// Returns the volume for a specific step as Decimal.
    fn get_volume(&mut self, step: usize) -> Decimal;
}

/// Constant volume model.
#[derive(Clone)]
pub struct ConstantVolume {
    /// The constant volume amount.
    pub amount: Amount,
    /// Volume as decimal for convenience.
    volume_decimal: Decimal,
}

impl ConstantVolume {
    /// Creates a new constant volume model from an Amount.
    #[must_use]
    pub fn from_amount(amount: Amount) -> Self {
        Self {
            amount,
            volume_decimal: amount.to_decimal(),
        }
    }

    /// Creates a new constant volume model from a Decimal.
    #[must_use]
    pub fn new(volume: Decimal) -> Self {
        Self {
            amount: Amount::from_decimal(volume, 6),
            volume_decimal: volume,
        }
    }
}

impl VolumeModel for ConstantVolume {
    fn next_volume(&mut self) -> Amount {
        self.amount
    }

    fn get_volume(&mut self, _step: usize) -> Decimal {
        self.volume_decimal
    }
}

// Could add StochasticVolume later
