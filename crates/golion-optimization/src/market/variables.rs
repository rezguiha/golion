use crate::support::power_to_energy;
use golion_domain::{temporal::series::TimeStampedUtc, units::power::KiloWatt};
use good_lp::{Constraint, Expression, ProblemVariables, constraint, variable};
use jiff::{SignedDuration, Timestamp};

/// Bidding Variables container with time information
#[derive(Debug)]
pub struct BidVariables {
    start_at: Timestamp,
    input_power: Expression,
    output_power: Expression,
    input_energy: Expression,
    output_energy: Expression,
}
impl BidVariables {
    pub fn new(
        start_at: Timestamp,
        input_power: Expression,
        output_power: Expression,
        grid_step: &SignedDuration,
    ) -> Self {
        let output_energy = power_to_energy(&output_power, grid_step);
        let input_energy = power_to_energy(&input_power, grid_step);
        Self { start_at, input_power, output_power, input_energy, output_energy }
    }
    pub fn input_power(&self) -> &Expression {
        &self.input_power
    }
    pub fn output_power(&self) -> &Expression {
        &self.output_power
    }
    pub fn input_energy(&self) -> &Expression {
        &self.input_energy
    }
    pub fn output_energy(&self) -> &Expression {
        &self.output_energy
    }
    /// Set Exclusivity between input and output power with big M constraint.
    /// This is needed for wholesale market biding for example.
    pub fn exclusivity_constraint(
        &self,
        vars: &mut ProblemVariables,
        big_m_input: KiloWatt,
        big_m_output: KiloWatt,
    ) -> [Constraint; 2] {
        let exclusivity_binary = vars.add(variable().binary());
        [
            constraint!(self.input_power.clone() <= exclusivity_binary * big_m_input.0),
            constraint!(
                self.output_power.clone() <= (1 - exclusivity_binary) * big_m_output.0
            ),
        ]
    }
}

// Implement TimeStampedUtc to enable creation
// of TimeSeries<BidVariables> out of Vec<BidVariables>.
impl TimeStampedUtc for BidVariables {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}
