/// Asset type static specifications
/// This defines mainly technical constructor information
/// and user defined usage limitations.
use garde::Validate;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
/// Bess static specifications.
#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder, Clone)]
pub struct BessSpecs {
    /// Theoretical capacity expressed in kWh in grid side convention.
    #[builder(default = 7500.0)]
    #[garde(range(min = 0.0))]
    rated_energy: f32,
    /// Theoretical maximum charge power in kW in grid side convention.
    #[builder(default = 2500.0)]
    #[garde(range(min = 0.0))]
    rated_charge_power: f32,
    /// Theoretical maximum discharge power in kW in grid side convention.
    #[builder(default = 2500.0)]
    #[garde(range(min = 0.0))]
    rated_discharge_power: f32,
    /// The charging efficiency expressed in percentage in \[0,1\]
    #[builder(default = 0.98)]
    #[garde(range(min = 0.0, max = 1.0))]
    charge_efficiency: f32,
    /// The discharging efficiency expressed in percentage in \[0,1\]
    #[builder(default = 0.97)]
    #[garde(range(min = 0.0, max = 1.0))]
    discharge_efficiency: f32,
    /// Minimal state of charge to not go under expressed in \[0,1\] and represents
    /// the energy stored in available modules divided by capacity of those available
    /// modules.
    #[builder(default = 0.95)]
    #[garde(range(min = 0.0, max = 1.0))]
    soc_min: f32,
    /// Maximal state of charge with same convention as soc_max
    #[builder(default = 0.05)]
    #[garde(range(min = 0.0, max = 1.0))]
    soc_max: f32,
}

/// Renewable asset specifications
#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder, Clone)]
pub struct RenewableSpecs {
    /// Nominal power output for renewable asset.
    /// This is a minimal implementation serving just
    /// for the skeleton of the generic architecture.
    #[builder(default = 2500.0)]
    #[garde(range(min = 0.0))]
    rated_power: f32,
}

/// Renewable asset specifications
#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder, Clone)]
pub struct CcgtSpecs {
    /// Nominal power output for Gas Turbine asset.
    /// /// This is a minimal implementation serving just
    /// for the skeleton of the generic architecture.
    #[builder(default = 2500.0)]
    #[garde(range(min = 0.0))]
    rated_power: f32,
}
