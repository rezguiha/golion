use crate::support::power_to_energy;
use golion_domain::temporal::series::TimeStampedUtc;
use good_lp::Expression;
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
}

// Implement TimeStampedUtc to enable creation
// of TimeSeries<BidVariables> out of Vec<BidVariables>.
impl TimeStampedUtc for BidVariables {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}
