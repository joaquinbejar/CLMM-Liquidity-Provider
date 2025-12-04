use amm_domain::value_objects::amount::Amount;

pub trait VolumeModel {
    fn next_volume(&mut self) -> Amount;
}

#[derive(Clone)]
pub struct ConstantVolume {
    pub amount: Amount,
}

impl VolumeModel for ConstantVolume {
    fn next_volume(&mut self) -> Amount {
        self.amount
    }
}

// Could add StochasticVolume later
