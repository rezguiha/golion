use crate::support::power_to_energy;
use chrono::Duration;
use good_lp::{Expression, ProblemVariables, Variable, variable};

// region: --- Asset Variables
/// Asset type variable definitions.
///
/// # Examples
///
/// ```
/// use golion_optimization::variables::core::BatteryVars;
/// use good_lp::{ProblemVariables, Solution, SolverModel, default_solver};
/// use chrono::Duration;
///
/// let mut vars = ProblemVariables::new();
/// // 100 kW discharge / 80 kW charge, SoC bounded to [10 %, 90 %].
/// let battery = BatteryVars::new(&mut vars, 100.0, 80.0, 0.1, 0.9);
///
/// let discharge = battery.output_energy(Duration::minutes(15));
///
/// // Unconstrained maximum: 100 kW × 0.25 h = 25 kWh.
/// let solution = vars.maximise(&discharge).using(default_solver).solve().unwrap();
/// assert_eq!(solution.eval(&discharge), 25.0);
/// ```
#[derive(Debug)]
pub struct BatteryVars {
    /// Discharge output power expressed in kW.
    pub output_power: Variable,
    /// Charge input power expressed in kW.
    pub input_power: Variable,
    /// State of charge variable expressed in percentage \[0,1\]
    pub soc: Variable,
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
        power_to_energy(self.output_power, duration)
    }
    #[inline]
    pub fn input_energy(&self, duration: Duration) -> Expression {
        power_to_energy(self.input_power, duration)
    }
}

#[derive(Debug)]
pub struct GeneratorVars {
    output_power: Variable,
}
impl GeneratorVars {
    pub fn new(vars: &mut ProblemVariables, max_output_power: f64) -> Self {
        Self { output_power: vars.add(variable().min(0).max(max_output_power)) }
    }
    #[inline]
    fn output_energy(&self, duration: Duration) -> Expression {
        power_to_energy(self.output_power, duration)
    }
}
// endregion: --- Asset Variables

// region: --- Market Variables

/// Bid variables for a single market slot.
///
/// `sell_power` and `buy_power` are constrained to discrete multiples of
/// `increment`, enforced by the underlying integer LP variable.
///
/// # Examples
///
/// ```
/// use golion_optimization::variables::core::BiddingVars;
/// use good_lp::{ProblemVariables, Solution, SolverModel, default_solver};
/// use chrono::Duration;
///
/// let mut vars = ProblemVariables::new();
/// // 500 kW sell/buy capacity, 10 kW increment → 50 discrete steps.
/// let bidding = BiddingVars::new(&mut vars, 505.0, 500.0, 10.0);
///
/// // Maximise sell power: 50 steps × 10 kW = 500 kW.
/// let solution = vars
///     .maximise(&bidding.sell_power)
///     .using(default_solver)
///     .solve()
///     .unwrap();
/// assert_eq!(solution.eval(&bidding.sell_power), 500.0);
///
/// // Sell volume over a 1-hour slot: 500 kW × 0.25 h = 500 kWh.
/// let sell_vol = bidding.sell_volume(Duration::minutes(15));
/// assert_eq!(solution.eval(&sell_vol), 125.0);
/// ```
#[derive(Debug)]
pub struct BiddingVars {
    /// Sell bid power in kW, expressed as a multiple of the increment.
    pub sell_power: Expression,
    /// Buy bid power in kW, expressed as a multiple of the increment.
    pub buy_power: Expression,
}
impl BiddingVars {
    pub fn new(
        vars: &mut ProblemVariables,
        max_sell_power: f64,
        max_buy_power: f64,
        increment: f64,
    ) -> Self {
        let sell_power_int = vars
            .add(variable().integer().min(0.0).max((max_sell_power / increment).floor()));
        let buy_power_int = vars
            .add(variable().integer().min(0.0).max((max_buy_power / increment).floor()));
        Self {
            sell_power: sell_power_int * increment,
            buy_power: buy_power_int * increment,
        }
    }
    #[inline]
    pub fn sell_volume(&self, duration: Duration) -> Expression {
        power_to_energy(&self.sell_power, duration)
    }
    #[inline]
    pub fn buy_volume(&self, duration: Duration) -> Expression {
        power_to_energy(&self.buy_power, duration)
    }
}

// endregion: --- Market Variables
