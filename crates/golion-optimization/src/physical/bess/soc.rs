/// State of Charge related constraints.
use crate::physical::variables::BessVariables;
use crate::support::power_to_energy;
use golion_domain::{temporal::step::MinuteStep, units::efficiency::Efficiency};
use good_lp::{Constraint, Expression, IntoAffineExpression, Variable, constraint};
#[derive(Debug)]
pub(super) struct PreviousSoc {
    pub(super) upper: Expression,
    pub(super) nominal: Expression,
    pub(super) lower: Expression,
}
pub(super) fn transition_constraint(
    current_step_input_power: &Variable,
    current_step_output_power: &Variable,
    current_step_soc: &Variable,
    previous_step_soc: Expression,
    step: &MinuteStep,
    charge_efficiency: &Efficiency,
    discharge_efficiency: &Efficiency,
) -> Constraint {
    constraint!(
        current_step_soc.into_expression()
            == previous_step_soc
                + power_to_energy(
                    current_step_input_power.into_expression()
                        * *charge_efficiency.value()
                        - current_step_output_power.into_expression()
                            * (1.0_f64 / discharge_efficiency.value()),
                    step.duration()
                )
    )
}

pub(super) fn transition_constraint_scenarios(
    current_step_variables: &BessVariables,
    previous_step_soc: PreviousSoc,
    step: &MinuteStep,
    charge_efficiency: &Efficiency,
    discharge_efficiency: &Efficiency,
) -> [Constraint; 3] {
    [
        transition_constraint(
            &current_step_variables.input_power,
            &current_step_variables.output_power,
            &current_step_variables.soc,
            previous_step_soc.nominal,
            step,
            charge_efficiency,
            discharge_efficiency,
        ),
        transition_constraint(
            &current_step_variables.input_power_worst_upper,
            &current_step_variables.output_power_worst_upper,
            &current_step_variables.soc_worst_upper,
            previous_step_soc.upper,
            step,
            charge_efficiency,
            discharge_efficiency,
        ),
        transition_constraint(
            &current_step_variables.input_power_worst_lower,
            &current_step_variables.output_power_worst_lower,
            &current_step_variables.soc_worst_lower,
            previous_step_soc.lower,
            step,
            charge_efficiency,
            discharge_efficiency,
        ),
    ]
}
