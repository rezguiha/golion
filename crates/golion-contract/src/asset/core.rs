/// Different type asset definition.
use super::availability::{GeneratorAvailability, StorageAvailability};
use super::specifications::{BessSpecs, CcgtSpecs, RenewableSpecs};
use crate::market::choice::MarketChoice;
use garde::Validate;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use uuid::Uuid;

// region: Asset Identification
#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder)]
pub struct AssetIdentification {
    /// The asset unique identification.
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    pub asset_id: Uuid,
    /// The connection point identification which will serve
    /// as a grouping to model grid facing limitations/constraints.
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    pub connection_point_id: Uuid,
    /// The balancing service provider id which will serve as a grouping
    /// to model ancillary services facing interface. (FCR, aFRR, mFRR)
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    pub bsp_id: Uuid,

    /// The balancing role party id which will serve as a grouping
    /// to model imbalance facing interface.(Day ahead, intraday ,imbalance)
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    pub brp_id: Uuid,
}
// endregion: Asset Identification

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
    pub availability: GeneratorAvailability,
    /// Physical specification for Gas Turbine
    #[garde(dive)]
    pub specs: CcgtSpecs,
    /// Market configuration and corresponding data.
    #[garde(skip)]
    pub market_choices: Vec<MarketChoice>,
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
}
// endregion: All asset types' physical input data.

// region: Asset Enumeration Definition
/// Discriminated union of all asset types, tagged by `asset_type` in JSON.
///
/// # Examples
///
/// ```
/// use golion_contract::asset::availability::StorageAvailability;
/// use golion_contract::asset::core::{AssetData, BessData, AssetIdentification};
/// use golion_contract::asset::specifications::BessSpecs;
/// use golion_domain::{market::market_type::WholesaleMarketType,countries::Countries};
/// use golion_contract::market::choice::MarketChoice;
/// use chrono::Utc;
/// let market_choice = MarketChoice::WholeSaleChoice {
///             market: WholesaleMarketType::SpotDayAhead,
///             country: Countries::FR,
///         };
/// let availability = vec![
///     StorageAvailability::builder()
///         .start_at(Utc::now())
///         .max_charge_power(20.0)
///         .max_discharge_power(20.0)
///         .max_usable_energy(100.0)
///         .build()
/// ];
/// let asset = AssetData::Bess(
///     BessData::builder()
///         .availability(availability
///         )
///         .specs(BessSpecs::builder().build())
///         .identification(AssetIdentification::builder().build())
///         .market_choices(vec![market_choice])
///         .build()
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
