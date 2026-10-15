use crate::physical::{ActivationEnergy, variables::BessVariables};
/// Cycling constraints related logic.
use golion_domain::{
    asset::bess::{
        efficiency::BessPowerEfficiencies,
        specification::BessSpecifications,
        throughput::{BidirectionalThroughput, DischargeThroughput, ThroughputHistory},
    },
    temporal::{
        series::{TimeSeries, TimeStampedUtc},
        step::MinuteStep,
    },
    units::power::KiloWattHour,
};
use good_lp::{Constraint, Expression, IntoAffineExpression, constraint};
use jiff::ToSpan;

pub trait ComputeCycleContribution {
    fn modelled_throughput(&self, efficiencies: &BessPowerEfficiencies) -> f64;
    fn modelled_throughput_expression(
        vars: &BessVariables,
        expected_activation: &ActivationEnergy,
        step: &MinuteStep,
        efficiencies: &BessPowerEfficiencies,
    ) -> Expression;
    fn throughput_limit(usable_energy: KiloWattHour, max_24h_avg_cycles: f64) -> f64;
}
impl ComputeCycleContribution for BidirectionalThroughput {
    fn modelled_throughput(&self, efficiencies: &BessPowerEfficiencies) -> f64 {
        self.charged_energy.0 * efficiencies.charge_efficiency.value()
            + self.discharged_energy.0 / efficiencies.discharge_efficiency.value()
    }
    fn modelled_throughput_expression(
        vars: &BessVariables,
        expected_activation: &ActivationEnergy,
        step: &MinuteStep,
        efficiencies: &BessPowerEfficiencies,
    ) -> Expression {
        (vars.input_power * step.hours_f64() + &expected_activation.downward)
            * *efficiencies.charge_efficiency.value()
            + (vars.output_power * step.hours_f64() + &expected_activation.upward)
                / *efficiencies.discharge_efficiency.value()
    }
    fn throughput_limit(usable_energy: KiloWattHour, max_24h_avg_cycles: f64) -> f64 {
        2.0 * usable_energy.0 * max_24h_avg_cycles
    }
}
impl ComputeCycleContribution for DischargeThroughput {
    fn modelled_throughput(&self, efficiencies: &BessPowerEfficiencies) -> f64 {
        self.discharged_energy.0 / *efficiencies.discharge_efficiency.value()
    }
    fn modelled_throughput_expression(
        vars: &BessVariables,
        expected_activation: &ActivationEnergy,
        step: &MinuteStep,
        efficiencies: &BessPowerEfficiencies,
    ) -> Expression {
        (vars.output_power * step.hours_f64() + &expected_activation.upward)
            / *efficiencies.discharge_efficiency.value()
    }
    fn throughput_limit(usable_energy: KiloWattHour, max_24h_avg_cycles: f64) -> f64 {
        usable_energy.0 * max_24h_avg_cycles
    }
}

fn compute_throughput_constraints<C: ComputeCycleContribution + TimeStampedUtc>(
    past_series: &TimeSeries<C>,
    expected_activation_series: &TimeSeries<ActivationEnergy>,
    variable_store: &TimeSeries<BessVariables>,
    usable_energy: KiloWattHour,
    efficiencies: &BessPowerEfficiencies,
    max_24h_avg_cycles: f64,
) -> crate::Result<Vec<Constraint>> {
    let mut constraints =
        Vec::<Constraint>::with_capacity(variable_store.grid().length());
    let grid_start = *variable_store.grid().start();
    let step = variable_store.grid().step();
    for dt in variable_store.grid().iter() {
        let start_consideration = dt - 24.hours() + step.span();
        let start_future_consideration =
            start_consideration.max(*variable_store.grid().start());
        let relevant_past_throughput = past_series
            .data()
            .iter()
            .filter(|throughput_row| {
                throughput_row.start_at() >= &start_consideration
                    && throughput_row.start_at() < &grid_start
            })
            .map(|throughput_row| throughput_row.modelled_throughput(efficiencies))
            .fold(0.0, |acc, x| acc + x);
        let current_element_index = variable_store.grid().index_of(&dt)?;
        let start_element_index =
            variable_store.grid().index_of(&start_future_consideration)?;
        let relevant_future_throughput = variable_store.data()
            [start_element_index..=current_element_index]
            .iter()
            .zip(
                &expected_activation_series.data()
                    [start_element_index..=current_element_index],
            )
            .fold(0.0.into_expression(), |acc, (vars, expected_activation)| {
                acc + C::modelled_throughput_expression(
                    vars,
                    expected_activation,
                    step,
                    efficiencies,
                )
            });
        constraints.push(constraint!(
            // Make sure in case past throughput exceeds limit
            // We get no infeasible solution.
            relevant_future_throughput
                <= (C::throughput_limit(usable_energy, max_24h_avg_cycles)
                    - relevant_past_throughput)
                    .max(0.0)
        ));
    }
    Ok(constraints)
}

pub(crate) fn create_cycling_constraints(
    specifications: &BessSpecifications,
    variable_store: &TimeSeries<BessVariables>,
    expected_activation_series: &TimeSeries<ActivationEnergy>,
) -> crate::Result<Vec<Constraint>> {
    match &specifications.cycling.history {
        ThroughputHistory::Bidirectional(past_series) => compute_throughput_constraints(
            past_series,
            expected_activation_series,
            variable_store,
            specifications.rated_usable_energy(),
            &specifications.efficiencies,
            specifications.cycling.max_24h_avg_cycles,
        ),
        ThroughputHistory::DischargeOnly(past_series) => compute_throughput_constraints(
            past_series,
            expected_activation_series,
            variable_store,
            specifications.rated_usable_energy(),
            &specifications.efficiencies,
            specifications.cycling.max_24h_avg_cycles,
        ),
    }
}
