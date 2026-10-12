/// Past Throughput on batteries.
use garde::Validate;
use golion_domain::asset::bess::throughput::{
    BidirectionalThroughput, DischargeThroughput, ThroughputHistory,
};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder, Clone, Copy)]
pub struct BidirectionalThroughputContract {
    #[garde(skip)]
    pub start_at: Timestamp,
    /// Cumulated charged energy over time step in kWh
    /// measure at the grid side.
    #[garde(range(min = 0.0))]
    pub charged_energy: f64,
    /// Cumulated discharged energy over time step in kWh
    /// measured at the grid side.
    #[garde(range(min = 0.0))]
    pub discharged_energy: f64,
}
#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder, Clone, Copy)]
pub struct DischargeThroughputContract {
    #[garde(skip)]
    pub start_at: Timestamp,
    /// Cumulated discharged energy over time step in kWh
    /// measured at the grid side.
    #[garde(range(min = 0.0))]
    pub discharged_energy: f64,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
#[serde(untagged)]
pub enum ThroughputData {
    #[garde(dive)]
    // Make sure Bidirectional is first so serde does not mistake bidirectional
    // throughput as dischargeonly as it is same struct with extra attribute.
    Bidirectional(#[garde(dive)] Vec<BidirectionalThroughputContract>),
    DischargeOnly(#[garde(dive)] Vec<DischargeThroughputContract>),
}

// region: Domain Conversion
impl From<&DischargeThroughputContract> for DischargeThroughput {
    fn from(value: &DischargeThroughputContract) -> Self {
        Self {
            start_at: value.start_at,
            discharged_energy: value.discharged_energy.into(),
        }
    }
}
impl From<&BidirectionalThroughputContract> for BidirectionalThroughput {
    fn from(value: &BidirectionalThroughputContract) -> Self {
        Self {
            start_at: value.start_at,
            discharged_energy: value.discharged_energy.into(),
            charged_energy: value.charged_energy.into(),
        }
    }
}

impl TryFrom<&ThroughputData> for ThroughputHistory {
    type Error = crate::Error;
    fn try_from(value: &ThroughputData) -> crate::Result<Self> {
        Ok(match value {
            ThroughputData::Bidirectional(values) => ThroughputHistory::Bidirectional(
                values
                    .iter()
                    .map(BidirectionalThroughput::from)
                    .collect::<Vec<_>>()
                    .try_into()?,
            ),
            ThroughputData::DischargeOnly(values) => ThroughputHistory::DischargeOnly(
                values
                    .iter()
                    .map(DischargeThroughput::from)
                    .collect::<Vec<_>>()
                    .try_into()?,
            ),
        })
    }
}
// endregion: Domain Conversion
