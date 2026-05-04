use crate::optimization::support::power_to_energy;
use chrono::Duration;
use good_lp::{Expression, ProblemVariables, Variable, variable};

// region: --- Asset Variables
/// Asset type variable definitions.
#[derive(Debug)]
pub struct BatteryVars {
    /// Discharge output power expressed in kW.
    output_power: Variable,
    /// Charge input power expressed in kW.
    input_power: Variable,
    /// State of charge variable expressed in percentage \[0,1\]
    soc: Variable,
}
impl BatteryVars {
    pub fn new(
        vars: &mut ProblemVariables,
        max_output_power: f64,
        max_input_power: f64,
        min_soc: f64,
        max_soc: f64,
    ) -> Self {
        Self {
            output_power: vars.add(variable().min(0).max(max_output_power)),
            input_power: vars.add(variable().min(0).max(max_input_power)),
            soc: vars.add(variable().min(min_soc).max(max_soc)),
        }
    }
    #[inline]
    pub fn output_energy(&self, duration: Duration) -> Expression {
        power_to_energy(&self.output_power, duration)
    }
    #[inline]
    pub fn input_energy(&self, duration: Duration) -> Expression {
        power_to_energy(&self.input_power, duration)
    }
}

#[derive(Debug)]
pub struct GeneratorVars {
    output_power: Variable,
}
impl GeneratorVars {
    pub fn new(vars: &mut ProblemVariables, max_output_power: f64) -> Self {
        Self {
            output_power: vars.add(variable().min(0).max(max_output_power)),
        }
    }
    #[inline]
    fn output_energy(&self, duration: Duration) -> Expression {
        power_to_energy(&self.output_power, duration)
    }
}
// endregion: --- Asset Variables

// region: --- Market Variables
/// This is still a work in progress and needs to be developed more to handle
/// bidding variables that can be expressed in power in kW or in energy in kWh.
#[derive(Debug)]
pub struct BiddingVars {
    sell: Variable,
    buy: Variable,
}

// endregion: --- Market Variables
