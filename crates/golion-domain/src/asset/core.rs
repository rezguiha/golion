/// Different type asset definition.
use super::availability::{GeneratorAvailability, StorageAvailability};
use super::specifications::{BessSpecs, CcgtSpecs, RenewableSpecs};
use garde::Validate;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use uuid::Uuid;
// region: All asset types' physical input data.

#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder)]
pub struct AssetIdentification {
    /// The asset unique identification.
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    asset_id: Uuid,
    /// The connection point identification which will serve
    /// as a grouping to model grid facing limitations/constraints.
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    connection_point_id: Uuid,
    /// The balancing service provider id which will serve as a grouping
    /// to model ancillary services facing interface. (FCR, aFRR, mFRR)
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    bsp_id: Uuid,

    /// The balancing role party id which will serve as a grouping
    /// to model imbalance facing interface.(Day ahead, intraday ,imbalance)
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    brp_id: Uuid,
}
/// Bess system representing a
/// unit on which we can collect scada data.
#[derive(Debug, Validate, Serialize, Deserialize, TypedBuilder)]
pub struct BessData {
    /// The asset unique identification id and grouping ids.
    #[serde(flatten)]
    #[garde(dive)]
    identification: AssetIdentification,
    /// The declared future availability level of the Bess system
    #[garde(dive)]
    availability: StorageAvailability,
    /// Physical specification for Bess
    #[garde(dive)]
    specs: BessSpecs,
}

/// Combined Cycle Gas Turbine unit on which we can collect scada
/// data.
#[derive(Debug, Validate, Serialize, Deserialize, TypedBuilder)]
pub struct GasTurbineData {
    /// The asset unique identification id and grouping ids.
    #[serde(flatten)]
    #[garde(dive)]
    identification: AssetIdentification,

    #[garde(dive)]
    /// The declared future availability level of the CCGT
    availability: GeneratorAvailability,
    /// Physical specification for Gas Turbine
    #[garde(dive)]
    specs: CcgtSpecs,
}

/// Renewable Asset unit on which we can collect scada data.
#[derive(Debug, Validate, Serialize, Deserialize, TypedBuilder)]
pub struct RenewableData {
    /// The asset unique identification id and grouping ids.
    #[serde(flatten)]
    #[garde(dive)]
    identification: AssetIdentification,

    #[garde(dive)]
    /// The declared future availability level of the CCGT
    availability: GeneratorAvailability,
    /// Physical specification for Gas Turbine
    #[garde(dive)]
    specs: RenewableSpecs,
}
// endregion: All asset types' physical input data.

// region: Asset Enumeration Definition
/// Discriminated union of all asset types, tagged by `asset_type` in JSON.
///
/// # Examples
///
/// ```
/// use golion_domain::asset::availability::StorageAvailability;
/// use golion_domain::asset::core::{AssetData, BessData, AssetIdentification};
/// use golion_domain::asset::specifications::BessSpecs;
/// use chrono::Utc;
/// let asset = AssetData::Bess(
///     BessData::builder()
///         .availability(
///             StorageAvailability::builder()
///                 .start_at(vec![Utc::now()])
///                 .max_charge_power(vec![2.5])
///                 .max_discharge_power(vec![2.4])
///                 .max_usable_energy(vec![7.5])
///                 .build(),
///         )
///         .specs(BessSpecs::builder().build())
///         .identification(AssetIdentification::builder().build())
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
