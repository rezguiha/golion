/// Different type asset definition.
use super::availability::{GeneratorAvailability, StorageAvailability};
use super::specifications::{BessSpecs, CcgtSpecs, RenewableSpecs};
use garde::Validate;
use serde::Deserialize;
use typed_builder::TypedBuilder;
use uuid::Uuid;
// region: All asset types' physical input data.

/// Bess system representing a
/// unit on which we can collect scada data.
#[derive(Debug, Validate, Deserialize, TypedBuilder)]
pub struct BessData {
    /// The asset unique identification.
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    asset_id: Uuid,
    /// The connection point identification which will serve
    /// as a grouping to model grid facing limitations/constraints.
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    connection_point_id: Uuid,
    /// The declared future availability level of the Bess system
    #[garde(dive)]
    availability: StorageAvailability,
    /// Physical specification for Bess
    #[garde(dive)]
    specs: BessSpecs,
}

/// Combined Cycle Gas Turbine unit on which we can collect scada
/// data.
#[derive(Debug, Validate, Deserialize, TypedBuilder)]
pub struct GasTurbineData {
    /// The asset unique identification.
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    asset_id: Uuid,
    /// The connection point identification which will serve
    /// as a grouping to model grid facing limitations/constraints.
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    connection_point_id: Uuid,
    #[garde(dive)]
    /// The declared future availability level of the CCGT
    availability: GeneratorAvailability,
    /// Physical specification for Gas Turbine
    #[garde(dive)]
    specs: CcgtSpecs,
}

/// Renewable Asset unit on which we can collect scada data.
#[derive(Debug, Validate, Deserialize, TypedBuilder)]
pub struct RenewableData {
    /// The asset unique identification.
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    asset_id: Uuid,
    /// The connection point identification which will serve
    /// as a grouping to model grid facing limitations/constraints.
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    connection_point_id: Uuid,
    #[garde(dive)]
    /// The declared future availability level of the CCGT
    availability: GeneratorAvailability,
    /// Physical specification for Gas Turbine
    #[garde(dive)]
    specs: RenewableSpecs,
}
