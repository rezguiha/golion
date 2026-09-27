/// Market Commitments models with their store definition.
use garde::Validate;
use golion_domain::market::{
    commitment::{EnergyCommitment, PowerCommitment},
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
    /// Positive in case of charge net position
    /// and negative otherwise.
    pub net_position: f64,
}

pub type AncillaryCommitments =
    Vec<MarketSeries<AncillaryMarketType, AncillaryCommitment>>;
pub type WholesaleCommitments =
    Vec<MarketSeries<WholesaleMarketType, WholesaleCommitment>>;

// region: Domain Conversion
impl From<&AncillaryCommitment> for PowerCommitment {
    fn from(value: &AncillaryCommitment) -> Self {
        Self {
            start_at: value.start_at,
            input: value.downward_power.into(),
            output: value.upward_power.into(),
        }
    }
}
impl From<&WholesaleCommitment> for EnergyCommitment {
    fn from(value: &WholesaleCommitment) -> Self {
        Self {
            start_at: value.start_at,
            input: value.net_position.max(0.0).into(),
            output: value.net_position.min(0.0).abs().into(),
        }
    }
}
impl From<PowerCommitment> for AncillaryCommitment {
    fn from(value: PowerCommitment) -> Self {
        Self {
            start_at: value.start_at,
            upward_power: value.output.0,
            downward_power: value.input.0,
        }
    }
}
impl From<EnergyCommitment> for WholesaleCommitment {
    fn from(value: EnergyCommitment) -> Self {
        Self { start_at: value.start_at, net_position: value.input.0 - value.output.0 }
    }
}

// endregion: Domain Conversion
