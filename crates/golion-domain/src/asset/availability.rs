/// Availability structs definition.
use chrono::{DateTime, Utc};
use garde::Validate;
use serde::Deserialize;
use typed_builder::TypedBuilder;

use crate::series::HasTimeStamps;

/// Storage unit availability that can both charge and discharge.
/// Represents declared planned availability for future dates/times.
#[derive(Debug, Deserialize, Validate, TypedBuilder)]
pub struct StorageAvailability {
    /// Reference datetime for future declared availability.
    #[garde(skip)]
    start_at: Vec<DateTime<Utc>>,
    /// Maximum charge power with grid convention expressed in kW.
    #[garde(inner(range(min = 0.0)))]
    max_charge_power: Vec<f32>,
    /// Maximum discharge power with grid convention expressed in kW.
    #[garde(inner(range(min = 0.0)))]
    max_discharge_power: Vec<f32>,
    /// Maximum usable energy expressed in kWh.
    /// Does not reflect energy level in battery — reflects maximum connected capacity.
    #[garde(inner(range(min = 0.0)))]
    max_usable_energy: Vec<f32>,
}
impl HasTimeStamps for StorageAvailability {
    fn timestamps(&self) -> &[DateTime<Utc>] {
        &self.start_at
    }
}
/// Generator unit availability that can only output power.
/// Represents declared planned availability for future dates/times.
#[derive(Debug, Deserialize, Validate, TypedBuilder)]
pub struct GeneratorAvailability {
    /// Reference datetime for future declared availability.
    #[garde(skip)]
    start_at: Vec<DateTime<Utc>>,
    /// Maximum output power with grid convention expressed in kW.
    #[garde(inner(range(min = 0.0)))]
    max_output_power: Vec<f32>,
    /// Minimum output power with grid convention expressed in kW.
    #[garde(inner(range(min = 0.0)))]
    min_output_power: Vec<f32>,
}
impl HasTimeStamps for GeneratorAvailability {
    fn timestamps(&self) -> &[DateTime<Utc>] {
        &self.start_at
    }
}
