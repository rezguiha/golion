/// Asset type static specifications
/// This defines mainly technical constructor information
/// and user defined usage limitations.
use garde::Validate;
use golion_domain::asset::bess::{efficiency::BessPowerEfficiencies, limits::SocRange};
use golion_domain::units::efficiency::Efficiency;
use golion_domain::units::soc::SocFraction;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

// region: Asset Specifications

/// Bess static specifications.
#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder, Clone, Copy)]
pub struct BessSpecs {
    /// Theoretical capacity expressed in kWh in grid side convention.
    #[builder(default = 7500.0)]
    #[garde(range(min = 0.0))]
    pub rated_energy: f64,
    /// Theoretical maximum charge power in kW in grid side convention.
    #[builder(default = 2500.0)]
    #[garde(range(min = 0.0))]
    pub rated_charge_power: f64,
    /// Theoretical maximum discharge power in kW in grid side convention.
    #[builder(default = 2500.0)]
    #[garde(range(min = 0.0))]
    pub rated_discharge_power: f64,
    /// The charging efficiency expressed in percentage in \[0,1\]
    #[builder(default = 0.98)]
    #[garde(range(min = 0.0, max = 1.0))]
    pub charge_efficiency: f64,
    /// The discharging efficiency expressed in percentage in \[0,1\]
    #[builder(default = 0.97)]
    #[garde(range(min = 0.0, max = 1.0))]
    pub discharge_efficiency: f64,
    /// Minimal state of charge to not go under expressed in \[0,1\] and represents
    /// the energy stored in available modules divided by capacity of those available
    /// modules.
    #[builder(default = 0.95)]
    #[garde(range(min = 0.0, max = 1.0))]
    pub soc_min: f64,
    /// Maximal state of charge with same convention as soc_max
    #[builder(default = 0.05)]
    #[garde(range(min = 0.0, max = 1.0))]
    pub soc_max: f64,
}

/// Renewable asset specifications
#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder, Clone)]
pub struct RenewableSpecs {
    /// Nominal power output for renewable asset.
    /// This is a minimal implementation serving just
    /// for the skeleton of the generic architecture.
    #[builder(default = 2500.0)]
    #[garde(range(min = 0.0))]
    pub rated_power: f64,
}

/// Renewable asset specifications
#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder, Clone)]
pub struct CcgtSpecs {
    /// Nominal power output for Gas Turbine asset.
    /// /// This is a minimal implementation serving just
    /// for the skeleton of the generic architecture.
    #[builder(default = 2500.0)]
    #[garde(range(min = 0.0))]
    pub rated_power: f64,
}
// endregion: Asset Specifications

// region: Domain Conversions
impl TryInto<SocRange> for BessSpecs {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<SocRange> {
        let min_soc_fraction: SocFraction = self.soc_min.try_into()?;
        let max_soc_fraction: SocFraction = self.soc_max.try_into()?;
        Ok(SocRange { min_soc: min_soc_fraction, max_soc: max_soc_fraction })
    }
}

impl TryInto<BessPowerEfficiencies> for BessSpecs {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<BessPowerEfficiencies> {
        let charge_efficiency: Efficiency = self.charge_efficiency.try_into()?;
        let discharge_efficiency: Efficiency = self.discharge_efficiency.try_into()?;
        Ok(BessPowerEfficiencies { charge_efficiency, discharge_efficiency })
    }
}
// endregion: Domain Conversions
