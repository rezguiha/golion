/// Physical Variables definition.
/// It includes also their creation trait.
use golion_domain::asset::bess::availability::Availability;
use golion_domain::asset::bess::limits::SocBounds;
use golion_domain::asset::state::{BessState, OtherAssetState};
use golion_domain::temporal::series::TimeStampedUtc;
use golion_domain::units::power::KiloWatt;
use good_lp::{
    Constraint, Expression, IntoAffineExpression, ProblemVariables, Variable, constraint,
    variable,
};
use jiff::Timestamp;
// region: Bess Variables
/// Bess Variables container with time information.
/// Active powers and state of charge follow the planned dispatch,
/// without any ancillary activation.
#[derive(Debug)]
pub struct BessVariables {
    pub(crate) start_at: Timestamp,
    /// Represents active charge power in kW.
    pub(crate) input_power: Variable,
    /// Represents active discharge power in kW.
    pub(crate) output_power: Variable,
    /// Represents state of charge in kWh with starting
    /// interval convention.
    pub(crate) soc: Variable,
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
        soc_bounds: &SocBounds,
        variable_generator: &mut ProblemVariables,
    ) -> crate::Result<Self> {
        Ok(Self {
            start_at: *dt,
            input_power: variable_generator
                .add(variable().min(0.0).max(avail_point.max_charge_power)),
            output_power: variable_generator
                .add(variable().min(0.0).max(avail_point.max_discharge_power)),
            soc: variable_generator
                .add(variable().min(soc_bounds.min).max(soc_bounds.max)),
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
    pub(crate) fn dispatch(&self) -> Expression {
        self.input_power.into_expression() - self.output_power.into_expression()
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
    ) -> [Constraint; 2] {
        let exclusivity_binary = vars.add(variable().binary());
        [
            constraint!(self.input_power <= exclusivity_binary * max_charge_power.0),
            constraint!(
                self.output_power <= (1 - exclusivity_binary) * max_discharge_power.0
            ),
        ]
    }
    /// Sets headroom constraints: the planned dispatch must leave enough power
    /// room, within the slot limits, to fully activate the reserved ancillary power.
    pub(super) fn headroom_constraint(
        &self,
        max_charge_power: &KiloWatt,
        max_discharge_power: &KiloWatt,
    ) -> [Constraint; 2] {
        [
            constraint!(
                self.output_ancillary.into_expression() - self.dispatch()
                    <= max_discharge_power.0
            ),
            constraint!(
                self.input_ancillary.into_expression() + self.dispatch()
                    <= max_charge_power.0
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
            dispatch: solution.eval(self.dispatch()).into(),
            start_at: self.start_at,
            soc: solution.value(self.soc).into(),
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
