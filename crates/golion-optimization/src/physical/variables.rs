/// Physical Variables definition.
/// It includes also their creation trait.
use crate::Result;
use crate::physical::bess::soc::transition_constraint;
use golion_domain::asset::bess::availability::Availability;
use golion_domain::asset::bess::limits::SocRange;
use golion_domain::asset::bess::{
    efficiency::BessPowerEfficiencies, specification::BessSpecifications,
};
use golion_domain::temporal::series::TimeSeries;
use golion_domain::temporal::series::TimeStampedUtc;
use golion_domain::temporal::step::MinuteStep;
use golion_domain::units::power::KiloWattHour;
use good_lp::{Constraint, Expression, ProblemVariables, Variable, variable};
use jiff::Timestamp;
// region: Bess Variables
/// Bess Variables container with time information
#[derive(Debug)]
pub struct BessVariables {
    pub start_at: Timestamp,
    pub input_power: Variable,
    pub output_power: Variable,
    pub soc: Variable,
}
// Implement TimeStampedUtc to enable creation
// of TimeSeries<BessVariables> out of Vec<BessVariables>.
impl TimeStampedUtc for BessVariables {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}

// endregion: Bess Variables

// region: BessVariableCreator
pub trait BessVariableCreator {
    fn efficiencies(&self) -> &BessPowerEfficiencies;
    fn soc_range(&self) -> &SocRange;
    fn availability(&self) -> &TimeSeries<Availability>;
    // Creates variables at a particular timestamp t.
    fn create_variables_at(
        &self,
        dt: &Timestamp,
        variable_generator: &mut ProblemVariables,
    ) -> Result<BessVariables> {
        let avail_point = self.availability().at(dt)?;
        Ok(BessVariables {
            start_at: *dt,
            input_power: variable_generator
                .add(variable().min(0.0).max(avail_point.max_charge_power)),
            output_power: variable_generator
                .add(variable().min(0.0).max(avail_point.max_discharge_power)),
            soc: variable_generator.add(
                variable()
                    .min(
                        self.soc_range()
                            .min_soc
                            .into_energy_kwh(&avail_point.max_usable_energy),
                    )
                    .max(
                        self.soc_range()
                            .max_soc
                            .into_energy_kwh(&avail_point.max_usable_energy),
                    ),
            ),
        })
    }
    fn create_variables_and_minimal_constraints(
        &self,
        time_index: &[Timestamp],
        vars: &mut ProblemVariables,
        initial_soc: KiloWattHour,
        step: MinuteStep,
    ) -> Result<(TimeSeries<BessVariables>, Vec<Constraint>)> {
        let time_index_length = time_index.len();
        // Initialize battery physical variables and constraints containers.
        let mut constraints: Vec<Constraint> = Vec::with_capacity(time_index_length);
        let mut variable_vec: Vec<BessVariables> = Vec::with_capacity(time_index_length);
        // Loop over time index ,create variables with their respective limits
        // and generate defining soc constraints.
        for (i, dt) in time_index.iter().enumerate() {
            // Create battery physical variables.
            let variables_at = self.create_variables_at(dt, vars)?;

            // Create soc transition constraints.
            let prev_soc: Expression = match i {
                0 => initial_soc.0.into(),
                _ => variable_vec[i - 1].soc.into(),
            };
            constraints.push(transition_constraint(
                &variables_at,
                prev_soc,
                &step,
                &self.efficiencies().charge_efficiency,
                &self.efficiencies().discharge_efficiency,
            ));
            variable_vec.push(variables_at);
        }
        let variable_store: TimeSeries<BessVariables> = variable_vec.try_into()?;

        Ok((variable_store, constraints))
    }
}

impl BessVariableCreator for BessSpecifications {
    fn efficiencies(&self) -> &BessPowerEfficiencies {
        &self.efficiencies
    }
    fn soc_range(&self) -> &SocRange {
        &self.limits.soc_range
    }
    fn availability(&self) -> &TimeSeries<Availability> {
        &self.limits.availability
    }
}
// endregion: BessVariableCreator
