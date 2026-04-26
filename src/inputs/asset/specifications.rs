/// Asset type static specifications
/// This defines mainly technical constructor information
/// and user defined usage limitations.
use garde::Validate;
use serde::Deserialize;

/// Bess static specifications.
#[derive(Debug, Deserialize, Validate)]
pub struct BessSpecs {
    /// Theoretical capacity expressed in kWh in grid side convention.
    #[garde(range(min = 0.0))]
    rated_energy: f32,
    /// Theoretical maximum charge power in kW in grid side convention.
    #[garde(range(min = 0.0))]
    rated_charge_power: f32,
    /// Theoretical maximum discharge power in kW in grid side convention.
    #[garde(range(min = 0.0))]
    rated_discharge_power: f32,
    /// The charging efficiency expressed in percentage in \[0,1\]
    #[garde(range(min = 0.0, max = 1.0))]
    charge_efficiency: f32,
    /// The discharging efficiency expressed in percentage in \[0,1\]
    #[garde(range(min = 0.0, max = 1.0))]
    discharge_efficiency: f32,
    /// Minimal state of charge to not go under expressed in \[0,1\] and represents
    /// the energy stored in available modules divided by capacity of those available
    /// modules.
    #[garde(range(min = 0.0, max = 1.0))]
    soc_min: f32,
    /// Maximal state of charge with same convention as soc_max
    #[garde(range(min = 0.0, max = 1.0))]
    soc_max: f32,
}

/// Renewable asset specifications
#[derive(Debug, Deserialize, Validate)]
pub struct RenewableSpecs {
    /// Nominal power output for renewable asset.
    /// This is a minimal implementation serving just
    /// for the skeleton of the generic architecture.
    #[garde(range(min = 0.0))]
    rated_power: f32,
}

/// Renewable asset specifications
#[derive(Debug, Deserialize, Validate)]
pub struct CcgtSpecs {
    /// Nominal power output for Gas Turbine asset.
    /// /// This is a minimal implementation serving just
    /// for the skeleton of the generic architecture.
    #[garde(range(min = 0.0))]
    rated_power: f32,
}
