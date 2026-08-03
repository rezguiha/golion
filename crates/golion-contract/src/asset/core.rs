/// Different type asset definition.
use super::availability::{GeneratorAvailability, StorageAvailability};
use super::identification::AssetIdentification;
use super::specifications::{BessSpecs, CcgtSpecs, RenewableSpecs};
use crate::market::choice::MarketChoice;
use crate::market::commitments::{AncillaryCommitments, WholesaleCommitments};
use garde::Validate;
use golion_domain::asset::bess::availability::Availability;
use golion_domain::asset::bess::limits::{BessLimits, SocRange};
use golion_domain::temporal::series::TimeSeries;
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
    /// Physical specification for Bess
    #[garde(dive)]
    pub specs: BessSpecs,
    /// Market configuration and corresponding data.
    #[garde(skip)]
    pub market_choices: Vec<MarketChoice>,
    #[garde(skip)]
    pub ancillary_commitments: AncillaryCommitments,
    #[garde(skip)]
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
    #[garde(skip)]
    pub ancillary_commitments: AncillaryCommitments,
    #[garde(skip)]
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
    #[garde(skip)]
    pub ancillary_commitments: AncillaryCommitments,
    #[garde(skip)]
    pub wholesale_commitments: WholesaleCommitments,
}
// endregion: All asset types' physical input data.

// region: Asset Enumeration Definition
/// Discriminated union of all asset types, tagged by `asset_type` in JSON.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
///
/// use chrono::Utc;
/// use golion_contract::asset::availability::StorageAvailability;
/// use golion_contract::asset::core::{AssetData, BessData};
/// use golion_contract::asset::identification::AssetIdentification;
/// use golion_contract::asset::specifications::BessSpecs;
/// use golion_contract::market::choice::MarketChoice;
/// use golion_contract::market::commitments::WholesaleCommitment;
/// use golion_domain::countries::Countries;
/// use golion_domain::market::market_type::WholesaleMarketType;
///
/// let market_choice = MarketChoice::WholeSaleChoice {
///     market: WholesaleMarketType::SpotDayAhead,
///     country: Countries::FR,
/// };
/// let availability = vec![
///     StorageAvailability::builder()
///         .start_at(Utc::now())
///         .max_charge_power(20.0)
///         .max_discharge_power(20.0)
///         .max_usable_energy(100.0)
///         .build(),
/// ];
/// let wholesale_commitments = HashMap::from([(
///     WholesaleMarketType::SpotDayAhead,
///     vec![WholesaleCommitment::builder().start_at(Utc::now()).net_position(10.0).build()],
/// )]);
/// let asset = AssetData::Bess(
///     BessData::builder()
///         .availability(availability)
///         .specs(BessSpecs::builder().build())
///         .identification(AssetIdentification::builder().build())
///         .market_choices(vec![market_choice])
///         .wholesale_commitments(wholesale_commitments)
///         .ancillary_commitments(HashMap::new())
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

impl TryInto<BessLimits> for BessData {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<BessLimits> {
        let soc_range: SocRange = self.specs.try_into()?;
        let availability_raw: TimeSeries<StorageAvailability> =
            self.availability.try_into()?;
        let availability: TimeSeries<Availability> = TimeSeries {
            grid: availability_raw.grid,
            data: availability_raw.data.iter().map(|x| x.into()).collect(),
        };
        Ok(BessLimits { soc_range, availability })
    }
}
