use crate::asset::throughput::ThroughputData;
use crate::optimization::output::BessStateOutput;

/// Different type asset definition.
use super::availability::{GeneratorAvailability, StorageAvailability};
use super::specifications::{BessSpecs, CcgtSpecs, RenewableSpecs};
use garde::Validate;
use golion_domain::asset::bess::availability::Availability as BessAvailability;
use golion_domain::asset::bess::efficiency::BessPowerEfficiencies;
use golion_domain::asset::bess::soc::SocRange;
use golion_domain::asset::bess::specification::BessSpecifications;
use golion_domain::asset::bess::throughput::{Cycling, ThroughputHistory};
use golion_domain::problem::definition::AssetDefinition;
use golion_domain::units::power::KiloWattHour;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use uuid::Uuid;

// region: All asset types' physical input data.
/// Bess system representing a
/// unit on which we can collect scada data.
#[derive(Debug, Validate, Serialize, Deserialize, TypedBuilder)]
pub struct BessData {
    /// The asset unique identification
    #[garde(skip)]
    pub asset_id: Uuid,
    /// The declared future availability level of the Bess system
    #[garde(dive)]
    pub availability: Vec<StorageAvailability>,
    /// Past 24h measured throughput on battery.
    #[garde(dive)]
    pub throughput: ThroughputData,
    /// The current state of charge of the battery in kWh, at the start
    /// of the optimization run.
    #[garde(range(min = 0.0))]
    pub initial_soc: f64,
    /// The current projected state that was computed by the previous
    /// optimization run.
    #[garde(skip)]
    pub current_state: BessStateOutput,
    /// Physical specification for Bess
    #[garde(dive)]
    pub specs: BessSpecs,
}

/// Combined Cycle Gas Turbine unit on which we can collect scada
/// data.
#[derive(Debug, Validate, Serialize, Deserialize, TypedBuilder)]
pub struct GasTurbineData {
    /// The asset unique identification
    #[garde(skip)]
    pub asset_id: Uuid,

    #[garde(dive)]
    /// The declared future availability level of the CCGT
    pub availability: Vec<GeneratorAvailability>,
    /// Physical specification for Gas Turbine
    #[garde(dive)]
    pub specs: CcgtSpecs,
}

/// Renewable Asset unit on which we can collect scada data.
#[derive(Debug, Validate, Serialize, Deserialize, TypedBuilder)]
pub struct RenewableData {
    /// The asset unique identification
    #[garde(skip)]
    pub asset_id: Uuid,

    #[garde(dive)]
    /// The declared future availability level of the CCGT
    pub availability: Vec<GeneratorAvailability>,
    /// Physical specification for Gas Turbine
    #[garde(dive)]
    pub specs: RenewableSpecs,
}
// endregion: All asset types' physical input data.

// region: Asset Enumeration Definition
/// Discriminated union of all asset types, tagged by `asset_type` in JSON.
///
/// # Examples
///
/// ```
/// use jiff::Timestamp;
/// use golion_contract::asset::availability::StorageAvailability;
/// use golion_contract::asset::core::{AssetData, BessData};
/// use golion_contract::asset::specifications::BessSpecs;
/// use golion_contract::asset::throughput::{
///     BidirectionalThroughputContract, ThroughputData,
/// };
/// use golion_domain::countries::Countries;
/// use uuid::Uuid;
///
/// let availability = vec![
///     StorageAvailability::builder()
///         .start_at(Timestamp::now())
///         .max_charge_power(20.0)
///         .max_discharge_power(20.0)
///         .max_usable_energy(100.0)
///         .build(),
/// ];
/// let throughput = ThroughputData::Bidirectional(vec![
///     BidirectionalThroughputContract::builder()
///         .start_at(Timestamp::now())
///         .charged_energy(0.0)
///         .discharged_energy(0.0)
///         .build(),
/// ]);
/// let asset = AssetData::Bess(
///     BessData::builder()
///         .availability(availability)
///         .throughput(throughput)
///         .initial_soc(50.0)
///         .specs(
///             BessSpecs::builder()
///                 .throughput_penalty(0.0)
///                 .max_rolling_cycles(1.0)
///                 .build(),
///         )
///         .asset_id(Uuid::new_v4())
///         .build(),
/// );
/// ```
#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(tag = "asset_type")]
pub enum AssetData {
    #[serde(rename = "BESS")]
    Bess(#[garde(dive)] BessData),
    #[serde(rename = "CCGT")]
    GasTurbine(#[garde(dive)] GasTurbineData),
    #[serde(rename = "RENEWABLE")]
    Renewable(#[garde(dive)] RenewableData),
}
// endregion: Asset Enumeration Definition

// region: Domain Conversion

impl TryFrom<&BessData> for BessSpecifications {
    type Error = crate::Error;
    fn try_from(value: &BessData) -> Result<Self, Self::Error> {
        let soc_range: SocRange = value.specs.try_into()?;
        let availability: Vec<BessAvailability> =
            value.availability.iter().map(BessAvailability::from).collect();
        let efficiencies: BessPowerEfficiencies = value.specs.try_into()?;
        let cycling = Cycling {
            history: ThroughputHistory::try_from(&value.throughput)?,
            max_24h_avg_cycles: value.specs.max_rolling_cycles,
            penalty: value.specs.throughput_penalty,
        };
        Ok(BessSpecifications {
            soc_range,
            availability: availability.try_into()?,
            cycling,
            efficiencies,
            rated_charge_power: value.specs.rated_charge_power.into(),
            rated_discharge_power: value.specs.rated_discharge_power.into(),
            rated_energy: value.specs.rated_energy.into(),
        })
    }
}

impl AssetData {
    /// Converts the asset payload into its id and domain definition.
    pub fn try_into_definition(self) -> crate::Result<(Uuid, AssetDefinition)> {
        match self {
            Self::Bess(data) => {
                let specifications = BessSpecifications::try_from(&data)?;
                let initial_soc = KiloWattHour(data.initial_soc);
                let definition = AssetDefinition::Bess {
                    specifications,
                    initial_soc,
                    current_state: data.current_state.into(),
                };
                Ok((data.asset_id, definition))
            }
            Self::GasTurbine(GasTurbineData { asset_id, .. })
            | Self::Renewable(RenewableData { asset_id, .. }) => {
                Err(crate::Error::UnsupportedAssetType { asset_id })
            }
        }
    }
}
