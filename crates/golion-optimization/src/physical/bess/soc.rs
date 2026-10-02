/// State of Charge related constraints.
use crate::physical::variables::{BessOperatingPoint, BessVariables};
use crate::support::power_to_energy;
use golion_domain::{temporal::step::MinuteStep, units::efficiency::Efficiency};
use good_lp::{Constraint, Expression, IntoAffineExpression, constraint};
pub(crate) struct PreviousSoc {
    pub(crate) nominal: Expression,
    pub(crate) downward: Expression,
    pub(crate) upward: Expression,
}
pub fn transition_constraint(
    current_operating_point: &BessOperatingPoint,
    previous_step_soc: impl IntoAffineExpression,
    step: &MinuteStep,
    charge_efficiency: &Efficiency,
    discharge_efficiency: &Efficiency,
) -> Constraint {
    constraint!(
        current_operating_point.soc
            == previous_step_soc.into_expression()
                + power_to_energy(
                    current_operating_point.input_power * *charge_efficiency.value()
                        - current_operating_point.output_power
                            * (1.0_f64 / discharge_efficiency.value()),
                    step.duration()
                )
    )
}
pub(crate) fn transition_constraint_scenarios(
    current_step_variables: &BessVariables,
    previous_step_soc: PreviousSoc,
    step: &MinuteStep,
    charge_efficiency: &Efficiency,
    discharge_efficiency: &Efficiency,
) -> [Constraint; 3] {
    [
        current_step_variables.nominal.transition_constraint(
            previous_step_soc.nominal,
            step,
            charge_efficiency,
            discharge_efficiency,
        ),
        current_step_variables.downward_activation.transition_constraint(
            previous_step_soc.downward,
            step,
            charge_efficiency,
            discharge_efficiency,
        ),
        current_step_variables.upward_activation.transition_constraint(
            previous_step_soc.upward,
            step,
            charge_efficiency,
            discharge_efficiency,
        ),
    ]
}
