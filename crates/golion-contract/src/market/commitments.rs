/// Market Commitments models with their store definition.
use garde::Validate;
use golion_domain::market::{
    commitment::Commitment,
    market_type::{AncillaryMarketType, WholesaleMarketType},
};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::market::series::MarketSeries;
#[derive(Debug, Deserialize, Serialize, Validate, TypedBuilder)]
pub struct AncillaryCommitment {
    #[garde(skip)]
    pub start_at: Timestamp,
    /// Discharge Power in kW
    #[garde(range(min = 0.0))]
    pub upward_power: f64,
    /// Charge Power in kW
    #[garde(range(min = 0.0))]
    pub downward_power: f64,
}

#[derive(Debug, Deserialize, Serialize, Validate, TypedBuilder)]
pub struct WholesaleCommitment {
    #[garde(skip)]
    pub start_at: Timestamp,
    #[garde(skip)]
    /// Net position in kWH
    pub net_position: f64,
}

pub type AncillaryCommitments =
    Vec<MarketSeries<AncillaryMarketType, AncillaryCommitment>>;
pub type WholesaleCommitments =
    Vec<MarketSeries<WholesaleMarketType, WholesaleCommitment>>;

// region: Domain Conversion
impl From<AncillaryCommitment> for Commitment {
    fn from(value: AncillaryCommitment) -> Self {
        Self {
            start_at: value.start_at,
            input_power: value.downward_power.into(),
            output_power: value.upward_power.into(),
        }
    }
}

// endregion: Domain Conversion
