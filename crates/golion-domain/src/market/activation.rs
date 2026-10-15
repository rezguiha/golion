#[derive(Debug, Clone, Copy)]
pub struct ExpectedActivationRatios {
    upward: f64,
    downward: f64,
}

impl ExpectedActivationRatios {
    pub fn try_new(upward: f64, downward: f64) -> crate::Result<Self> {
        if (0.0..=1.0).contains(&upward) & (0.0..=1.0).contains(&downward) {
            Ok(Self { upward, downward })
        } else {
            Err(crate::market::error::MarketError::InvalidActivationRatioError {
                upward,
                downward,
            }
            .into())
        }
    }
    pub fn upward(&self) -> f64 {
        self.upward
    }
    pub fn downward(&self) -> f64 {
        self.downward
    }
}
