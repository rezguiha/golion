/// Availability structs definition.
use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Storage unit availability that can both charge and discharge.
/// Represents declared planned availability for future dates/times.
#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder)]
pub struct StorageAvailability {
    /// Reference datetime for future declared availability.
    #[garde(skip)]
    start_at: DateTime<Utc>,
    /// Maximum charge power with grid convention expressed in kW.
    #[garde(range(min = 0.0))]
    max_charge_power: f64,
    /// Maximum discharge power with grid convention expressed in kW.
    #[garde(range(min = 0.0))]
    max_discharge_power: f64,
    /// Maximum usable energy expressed in kWh.
    /// Does not reflect energy level in battery — reflects maximum connected capacity.
    #[garde(range(min = 0.0))]
    max_usable_energy: f64,
}

/// Generator unit availability that can only output power.
/// Represents declared planned availability for future dates/times.
#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder)]
pub struct GeneratorAvailability {
    /// Reference datetime for future declared availability.
    #[garde(skip)]
    start_at: DateTime<Utc>,
    /// Maximum output power with grid convention expressed in kW.
    #[garde(range(min = 0.0))]
    max_output_power: f64,
    /// Minimum output power with grid convention expressed in kW.
    #[garde(range(min = 0.0))]
    min_output_power: f64,
}
