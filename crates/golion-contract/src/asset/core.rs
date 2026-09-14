/// Different type asset definition.
use super::availability::{GeneratorAvailability, StorageAvailability};
use super::identification::AssetIdentification;
use super::specifications::{BessSpecs, CcgtSpecs, RenewableSpecs};
use crate::market::choice::MarketChoice;
use crate::market::commitments::{AncillaryCommitments, WholesaleCommitments};
use garde::Validate;
use golion_domain::asset::bess::availability::Availability as BessAvailability;
use golion_domain::asset::bess::efficiency::BessPowerEfficiencies;
use golion_domain::asset::bess::limits::{BessLimits, SocRange};
use golion_domain::asset::bess::specification::BessSpecifications;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

// region: All asset types' physical input data.
/// Bess system representing a
/// unit on which we can collect scada data.
#[derive(Debug, Validate, Serialize, Deserialize, TypedBuilder)]
pub struct BessData {
    /// The asset unique identification id and grouping ids.
    #[serde(flatten)]
    #[garde(dive)]
    pub identification: AssetIdentification,
    /// The declared future availability level of the Bess system
    #[garde(dive)]
    pub availability: Vec<StorageAvailability>,
    /// The current state of charge of the battery in kWh, at the start
    /// of the optimization run.
    #[garde(range(min = 0.0))]
    pub initial_soc: f64,
    /// Physical specification for Bess
    #[garde(dive)]
    pub specs: BessSpecs,
    /// Market configuration and corresponding data.
    #[garde(skip)]
    pub market_choices: Vec<MarketChoice>,
    #[garde(dive)]
    pub ancillary_commitments: AncillaryCommitments,
    #[garde(dive)]
    pub wholesale_commitments: WholesaleCommitments,
}

/// Combined Cycle Gas Turbine unit on which we can collect scada
/// data.
#[derive(Debug, Validate, Serialize, Deserialize, TypedBuilder)]
pub struct GasTurbineData {
    /// The asset unique identification id and grouping ids.
    #[serde(flatten)]
    #[garde(dive)]
    pub identification: AssetIdentification,

    #[garde(dive)]
    /// The declared future availability level of the CCGT
    pub availability: Vec<GeneratorAvailability>,
    /// Physical specification for Gas Turbine
    #[garde(dive)]
    pub specs: CcgtSpecs,
    /// Market configuration and corresponding data.
    #[garde(skip)]
    pub market_choices: Vec<MarketChoice>,
    #[garde(dive)]
    pub ancillary_commitments: AncillaryCommitments,
    #[garde(dive)]
    pub wholesale_commitments: WholesaleCommitments,
}

/// Renewable Asset unit on which we can collect scada data.
#[derive(Debug, Validate, Serialize, Deserialize, TypedBuilder)]
pub struct RenewableData {
    /// The asset unique identification id and grouping ids.
    #[serde(flatten)]
    #[garde(dive)]
    pub identification: AssetIdentification,

    #[garde(dive)]
    /// The declared future availability level of the CCGT
    pub availability: Vec<GeneratorAvailability>,
    /// Physical specification for Gas Turbine
    #[garde(dive)]
    pub specs: RenewableSpecs,
    /// Market configuration and corresponding data.
    #[garde(skip)]
    pub market_choices: Vec<MarketChoice>,
    #[garde(dive)]
    pub ancillary_commitments: AncillaryCommitments,
    #[garde(dive)]
    pub wholesale_commitments: WholesaleCommitments,
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
/// use golion_contract::asset::identification::AssetIdentification;
/// use golion_contract::asset::specifications::BessSpecs;
/// use golion_contract::market::choice::MarketChoice;
/// use golion_contract::market::commitments::WholesaleCommitment;
/// use golion_contract::market::series::MarketSeries;
/// use golion_domain::countries::Countries;
/// use golion_domain::market::market_type::WholesaleMarketType;
///
/// let market_choice = MarketChoice{
///     market: WholesaleMarketType::SpotDayAhead.into(),
///     country: Countries::FR,
///     product_step_minutes:15,
///     product_increment_kw:10,
/// };
/// let availability = vec![
///     StorageAvailability::builder()
///         .start_at(Timestamp::now())
///         .max_charge_power(20.0)
///         .max_discharge_power(20.0)
///         .max_usable_energy(100.0)
///         .build(),
/// ];
/// let wholesale_commitments = vec![
///     MarketSeries::builder()
///         .market(WholesaleMarketType::SpotDayAhead)
///         .country(Countries::FR)
///         .values(vec![
///             WholesaleCommitment::builder().start_at(Timestamp::now()).net_position(10.0).build(),
///         ])
///         .build(),
/// ];
/// let asset = AssetData::Bess(
///     BessData::builder()
///         .availability(availability)
///         .initial_soc(50.0)
///         .specs(BessSpecs::builder().build())
///         .identification(AssetIdentification::builder().build())
///         .market_choices(vec![market_choice])
///         .wholesale_commitments(wholesale_commitments)
///         .ancillary_commitments(Vec::new())
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
        let efficiencies = BessPowerEfficiencies {
            charge_efficiency: value.specs.charge_efficiency.try_into()?,
            discharge_efficiency: value.specs.discharge_efficiency.try_into()?,
        };
        let limits = BessLimits { soc_range, availability: availability.try_into()? };
        Ok(BessSpecifications { limits, efficiencies })
    }
}
