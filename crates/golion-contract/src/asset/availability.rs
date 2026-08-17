/// Availability structs definition.
use garde::Validate;
use golion_domain::temporal::series::TimeStampedUtc;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
type DomainBessAvailability = golion_domain::asset::bess::availability::Availability;
// region: Availability structs
/// Storage unit availability that can both charge and discharge.
/// Represents declared planned availability for future dates/times.
#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder, Clone)]
pub struct StorageAvailability {
    /// Reference datetime for future declared availability.
    #[garde(skip)]
    pub start_at: Timestamp,
    /// Maximum charge power with grid convention expressed in kW.
    #[garde(range(min = 0.0))]
    pub max_charge_power: f64,
    /// Maximum discharge power with grid convention expressed in kW.
    #[garde(range(min = 0.0))]
    pub max_discharge_power: f64,
    /// Maximum usable energy expressed in kWh.
    /// Does not reflect energy level in battery — reflects maximum connected capacity.
    #[garde(range(min = 0.0))]
    pub max_usable_energy: f64,
}

/// Generator unit availability that can only output power.
/// Represents declared planned availability for future dates/times.
#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder, Clone)]
pub struct GeneratorAvailability {
    /// Reference datetime for future declared availability.
    #[garde(skip)]
    pub start_at: Timestamp,
    /// Maximum output power with grid convention expressed in kW.
    #[garde(range(min = 0.0))]
    pub max_output_power: f64,
    /// Minimum output power with grid convention expressed in kW.
    #[garde(range(min = 0.0))]
    pub min_output_power: f64,
}
// endregion: Availability structs

// region: Domain Conversion
// We are disabling this clippy rule as implementing from
// will create dependency of domain crate to contract crate
// which is against hexagonal architecture principles.
impl From<&StorageAvailability> for DomainBessAvailability {
    fn from(x: &StorageAvailability) -> Self {
        DomainBessAvailability {
            max_charge_power: x.max_charge_power.into(),
            max_discharge_power: x.max_discharge_power.into(),
            max_usable_energy: x.max_usable_energy.into(),
        }
    }
}

impl TimeStampedUtc for StorageAvailability {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}
// endregion: Domain Conversion
