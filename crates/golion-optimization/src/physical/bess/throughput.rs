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
        step::Step,
    },
    units::power::KiloWattHour,
};
use good_lp::{
    Constraint, Expression, IntoAffineExpression, ProblemVariables, Variable, constraint,
    variable,
};
use jiff::ToSpan;

pub trait ComputeCycleContribution {
    fn modelled_throughput(&self, efficiencies: &BessPowerEfficiencies) -> f64;
    fn modelled_throughput_expression(
        vars: &BessVariables,
        expected_activation: &ActivationEnergy,
        step: &Step,
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
        step: &Step,
        efficiencies: &BessPowerEfficiencies,
    ) -> Expression {
        (vars.input_power * step.total_hours_f64() + &expected_activation.downward)
            * efficiencies.charge_efficiency.value()
            + (vars.output_power * step.total_hours_f64() + &expected_activation.upward)
                / efficiencies.discharge_efficiency.value()
    }
    fn throughput_limit(usable_energy: KiloWattHour, max_24h_avg_cycles: f64) -> f64 {
        2.0 * usable_energy.0 * max_24h_avg_cycles
    }
}
impl ComputeCycleContribution for DischargeThroughput {
    fn modelled_throughput(&self, efficiencies: &BessPowerEfficiencies) -> f64 {
        self.discharged_energy.0 / efficiencies.discharge_efficiency.value()
    }
    fn modelled_throughput_expression(
        vars: &BessVariables,
        expected_activation: &ActivationEnergy,
        step: &Step,
        efficiencies: &BessPowerEfficiencies,
    ) -> Expression {
        (vars.output_power * step.total_hours_f64() + &expected_activation.upward)
            / efficiencies.discharge_efficiency.value()
    }
    fn throughput_limit(usable_energy: KiloWattHour, max_24h_avg_cycles: f64) -> f64 {
        usable_energy.0 * max_24h_avg_cycles
    }
}

/// Limits the modelled throughput over every rolling 24-hour window ending
/// with a grid slot.
///
/// We compute at each timestamp the throughput from the start of the optimization.
/// At a particular step t , we make the difference of the total at that time step
/// minus the total at the timestep at start of rolling 24 hour window.
///
/// This gives us exactly the throughput we want to account for and it reduces
/// the density of the matrix for solve which reduces solve time.
fn compute_throughput_constraints<C: ComputeCycleContribution + TimeStampedUtc>(
    vars: &mut ProblemVariables,
    past_series: &TimeSeries<C>,
    expected_activation_series: &TimeSeries<ActivationEnergy>,
    variable_store: &TimeSeries<BessVariables>,
    usable_energy: KiloWattHour,
    efficiencies: &BessPowerEfficiencies,
    max_24h_avg_cycles: f64,
) -> crate::Result<Vec<Constraint>> {
    let grid = variable_store.grid();
    let step = grid.step();
    let limit = C::throughput_limit(usable_energy, max_24h_avg_cycles);
    let mut constraints = Vec::<Constraint>::with_capacity(2 * grid.length());
    let mut since_start = Vec::<Variable>::with_capacity(grid.length());
    for (i, dt) in grid.iter().enumerate() {
        // since_start[i] = since_start[i - 1] + throughput of slot i.
        let mut sum = C::modelled_throughput_expression(
            &variable_store.data()[i],
            &expected_activation_series.data()[i],
            &step,
            efficiencies,
        );
        if let Some(previous) = since_start.last() {
            sum += *previous;
        }
        let total = vars.add(variable());
        constraints.push(constraint!(total == sum));
        since_start.push(total);

        // 24-hour window ending with slot i, split at the grid start.
        let window_start = dt - 24.hours() + step.span();
        let first = grid.index_of(&window_start.max(grid.start()))?;
        // Part on the grid, slots first..=i.
        let mut window_on_grid = total.into_expression();
        if let Some(before_first) = first.checked_sub(1) {
            window_on_grid -= since_start[before_first];
        }
        // Part before the grid, from the throughput history.
        let window_before_grid: f64 = past_series
            .data()
            .iter()
            .filter(|row| row.start_at() >= window_start && row.start_at() < grid.start())
            .map(|row| row.modelled_throughput(efficiencies))
            .sum();
        // Clamped at zero so a history already over the limit cannot make the
        // problem infeasible.
        constraints
            .push(constraint!(window_on_grid <= (limit - window_before_grid).max(0.0)));
    }
    Ok(constraints)
}

pub(crate) fn create_cycling_constraints(
    vars: &mut ProblemVariables,
    specifications: &BessSpecifications,
    variable_store: &TimeSeries<BessVariables>,
    expected_activation_series: &TimeSeries<ActivationEnergy>,
) -> crate::Result<Vec<Constraint>> {
    match &specifications.cycling.history {
        ThroughputHistory::Bidirectional(past_series) => compute_throughput_constraints(
            vars,
            past_series,
            expected_activation_series,
            variable_store,
            specifications.rated_usable_energy(),
            &specifications.efficiencies,
            specifications.cycling.max_24h_avg_cycles,
        ),
        ThroughputHistory::DischargeOnly(past_series) => compute_throughput_constraints(
            vars,
            past_series,
            expected_activation_series,
            variable_store,
            specifications.rated_usable_energy(),
            &specifications.efficiencies,
            specifications.cycling.max_24h_avg_cycles,
        ),
    }
}
