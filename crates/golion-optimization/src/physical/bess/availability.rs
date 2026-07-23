use golion_domain::asset::bess::availability::Availability;
use good_lp::{Constraint, Variable, constraint};
/// Define constraints resulting from availability.
pub trait AvailabilityConstraints {
    fn charge_power_constraint(&self, charge_power: Variable) -> Constraint;
    fn discharge_power_constraint(&self, discharge_power: Variable) -> Constraint;
    fn energy_available_at_t(&self, soc: Variable) -> Constraint;
}

/// Define constraints resulting from availability.
impl AvailabilityConstraints for Availability {
    #[inline]
    fn charge_power_constraint(&self, charge_power: Variable) -> Constraint {
        constraint!(charge_power <= self.max_charge_power.0)
    }
    #[inline]
    fn discharge_power_constraint(&self, discharge_power: Variable) -> Constraint {
        constraint!(discharge_power <= self.max_discharge_power.0)
    }
    #[inline]
    fn energy_available_at_t(&self, soc: Variable) -> Constraint {
        constraint!(soc <= self.max_usable_energy.0)
    }
}
