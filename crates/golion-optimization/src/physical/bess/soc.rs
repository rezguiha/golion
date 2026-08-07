/// State of Charge related constraints.
use crate::physical::variables::BessVariables;
use crate::support::power_to_energy;
use golion_domain::{temporal::step::MinuteStep, units::efficiency::Efficiency};
use good_lp::{Constraint, Expression, constraint};

pub fn transition_constraint(
    current_step_variables: &BessVariables,
    previous_step_soc: Expression,
    step: &MinuteStep,
    charge_efficiency: &Efficiency,
    discharge_efficiency: &Efficiency,
) -> Constraint {
    constraint!(
        current_step_variables.soc
            == previous_step_soc
                + power_to_energy(
                    current_step_variables.input_power * *charge_efficiency.value()
                        - current_step_variables.output_power
                            * (1.0_f64 / discharge_efficiency.value()),
                    step.duration()
                )
    )
}
