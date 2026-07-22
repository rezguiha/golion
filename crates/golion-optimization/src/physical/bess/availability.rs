use golion_domain::units::power::{KiloWatt, KiloWattHour};
use good_lp::{Constraint, Variable, constraint};

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
/// Define constraints resulting from availability.
impl Availability {
    #[inline]
    pub fn charge_power_constraint(&self, charge_power: Variable) -> Constraint {
        constraint!(charge_power <= self.max_charge_power.0)
    }
    #[inline]
    pub fn discharge_power_constraint(&self, discharge_power: Variable) -> Constraint {
        constraint!(discharge_power <= self.max_discharge_power.0)
    }
    #[inline]
    pub fn energy_available_at_t(&self, soc: Variable) -> Constraint {
        constraint!(soc <= self.max_usable_energy.0)
    }
}
