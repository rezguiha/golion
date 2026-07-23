use crate::units::power::{KiloWatt, KiloWattHour};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Availability {
    /// Maximum charge power with grid convention expressed in kW.
    pub max_charge_power: KiloWatt,
    /// Maximum discharge power with grid convention expressed in kW.
    pub max_discharge_power: KiloWatt,
    /// Maximum usable energy expressed in kWh.
    /// Does not reflect energy level in battery.
    /// reflects maximum connected capacity.
    pub max_usable_energy: KiloWattHour,
}
