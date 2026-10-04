use crate::support::power_to_energy;
/// Physical Variables definition.
/// It includes also their creation trait.
use golion_domain::asset::bess::availability::Availability;
use golion_domain::asset::bess::limits::SocRange;
use golion_domain::asset::state::{BessState, OtherAssetState};
use golion_domain::temporal::series::TimeStampedUtc;
use golion_domain::units::power::KiloWatt;
use golion_domain::{temporal::step::MinuteStep, units::efficiency::Efficiency};
use good_lp::{
    Constraint, Expression, IntoAffineExpression, ProblemVariables, Variable, constraint,
    variable,
};
use jiff::Timestamp;
// region: Bess Variables
#[derive(Debug)]
pub struct BessOperatingPoint {
    /// Represents active charge power in kW.
    pub(crate) input_power: Variable,
    /// Represents active discharge power in kW.
    pub(crate) output_power: Variable,
    /// Represents state of charge in kWh with starting
    /// interval convention.
    pub(crate) soc: Variable,
}
impl BessOperatingPoint {
    pub(crate) fn new(
        avail_point: &Availability,
        soc_range: &SocRange,
        variable_generator: &mut ProblemVariables,
    ) -> Self {
        Self {
            input_power: variable_generator
                .add(variable().min(0.0).max(avail_point.max_charge_power)),
            output_power: variable_generator
                .add(variable().min(0.0).max(avail_point.max_discharge_power)),
            soc: variable_generator.add(
                variable()
                    .min(
                        soc_range
                            .min_soc()
                            .into_energy_kwh(&avail_point.max_usable_energy),
                    )
                    .max(
                        soc_range
                            .max_soc()
                            .into_energy_kwh(&avail_point.max_usable_energy),
                    ),
            ),
        }
    }
    pub(crate) fn dispatch(&self) -> Expression {
        self.input_power.into_expression() - self.output_power.into_expression()
    }
    /// Sets exclusivity constraints between input power and output power.
    /// This is necessary to be able to apply the right efficiency to the active power
    /// of the battery during charge and during discharge.
    pub(crate) fn exclusivity_constraint(
        &self,
        vars: &mut ProblemVariables,
        max_charge_power: &KiloWatt,
        max_discharge_power: &KiloWatt,
    ) -> [Constraint; 2] {
        let exclusivity_binary = vars.add(variable().binary());
        [
            constraint!(self.input_power <= exclusivity_binary * max_charge_power.0),
            constraint!(
                self.output_power <= (1 - exclusivity_binary) * max_discharge_power.0
            ),
        ]
    }
    pub(crate) fn transition_constraint(
        &self,
        previous_step_soc: Expression,
        step: &MinuteStep,
        charge_efficiency: &Efficiency,
        discharge_efficiency: &Efficiency,
    ) -> Constraint {
        constraint!(
            self.soc
                == previous_step_soc
                    + power_to_energy(
                        self.input_power * *charge_efficiency.value()
                            - self.output_power
                                * (1.0_f64 / discharge_efficiency.value()),
                        step.duration()
                    )
        )
    }
}

/// Bess Variables container with time information
#[derive(Debug)]
pub struct BessVariables {
    pub(crate) start_at: Timestamp,
    /// Operating point with only wholesale markets
    /// and no ancillary activation.
    pub(crate) nominal: BessOperatingPoint,
    /// Operating point with full activation on ancillary
    /// downward commitments.
    pub(crate) downward_activation: BessOperatingPoint,
    /// Operating point with full activation on ancillary
    /// upward commitments.
    pub(crate) upward_activation: BessOperatingPoint,
    /// Represents ancillary portion of charge (downward)
    /// commitments and bids that asset can deliver.
    pub(crate) input_ancillary: Variable,
    /// Represents ancillary portion of discharge (upward)
    /// commitments and bids that asset can deliver.
    pub(crate) output_ancillary: Variable,
}
impl BessVariables {
    pub fn try_new(
        dt: &Timestamp,
        avail_point: &Availability,
        soc_range: &SocRange,
        variable_generator: &mut ProblemVariables,
    ) -> crate::Result<Self> {
        Ok(Self {
            start_at: *dt,
            nominal: BessOperatingPoint::new(avail_point, soc_range, variable_generator),
            upward_activation: BessOperatingPoint::new(
                avail_point,
                soc_range,
                variable_generator,
            ),
            downward_activation: BessOperatingPoint::new(
                avail_point,
                soc_range,
                variable_generator,
            ),
            input_ancillary: variable_generator.add(
                variable()
                    .min(0.0)
                    .max(avail_point.max_discharge_power + avail_point.max_charge_power),
            ),
            output_ancillary: variable_generator.add(
                variable()
                    .min(0.0)
                    .max(avail_point.max_discharge_power + avail_point.max_charge_power),
            ),
        })
    }
}
// Implement Constraints on BessVariables
impl BessVariables {
    /// Sets exclusivity constraints between input power and output power.
    /// This is necessary to be able to apply the right efficiency to the active power
    /// of the battery during charge and during discharge.
    pub(super) fn exclusivity_constraint(
        &self,
        vars: &mut ProblemVariables,
        max_charge_power: &KiloWatt,
        max_discharge_power: &KiloWatt,
    ) -> impl Iterator<Item = Constraint> {
        self.nominal
            .exclusivity_constraint(vars, max_charge_power, max_discharge_power)
            .into_iter()
            .chain(self.downward_activation.exclusivity_constraint(
                vars,
                max_charge_power,
                max_discharge_power,
            ))
            .chain(self.upward_activation.exclusivity_constraint(
                vars,
                max_charge_power,
                max_discharge_power,
            ))
    }
    /// Worstcase-Nominal Link constraint
    pub(super) fn worstcase_nominal_link_constraint(&self) -> [Constraint; 2] {
        [
            constraint!(
                self.downward_activation.dispatch()
                    == self.nominal.dispatch() + self.input_ancillary
            ),
            constraint!(
                self.upward_activation.dispatch()
                    == self.nominal.dispatch() - self.output_ancillary
            ),
        ]
    }
}
// Implement TimeStampedUtc to enable creation
// of TimeSeries<BessVariables> out of Vec<BessVariables>.
impl TimeStampedUtc for BessVariables {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}
// Implement Domain Conversion

impl BessVariables {
    pub(crate) fn to_domain_state(&self, solution: &impl good_lp::Solution) -> BessState {
        BessState {
            dispatch: solution.eval(self.nominal.dispatch()).into(),
            start_at: self.start_at,
            soc: solution.value(self.nominal.soc).into(),
            soc_downward_activation: solution.value(self.downward_activation.soc).into(),
            soc_upward_activation: solution.value(self.upward_activation.soc).into(),
        }
    }
}
// endregion: Bess Variables

// region: OtherAsset variables

/// This is a temporary implementation of variables
/// of types other than BESS.
#[derive(Debug)]
pub struct OtherAssetVariables {
    pub(crate) start_at: Timestamp,
    pub(crate) output_power: Variable,
    /// Represents ancillary portion of discharge (upward)
    /// commitments and bids that asset can deliver.
    pub(crate) output_ancillary: Variable,
}
impl OtherAssetVariables {
    pub(crate) fn to_domain_state(
        &self,
        solution: &impl good_lp::Solution,
    ) -> OtherAssetState {
        OtherAssetState {
            start_at: self.start_at,
            dispatch: solution.value(self.output_power).into(),
        }
    }
}

// Implement TimeStampedUtc to enable creation
// of TimeSeries<BessVariables> out of Vec<BessVariables>.
impl TimeStampedUtc for OtherAssetVariables {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}

// endregion: OtherAsset variables
