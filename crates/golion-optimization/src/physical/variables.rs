/// Physical Variables definition.
/// It includes also their creation trait.
use crate::Result;
use chrono::{DateTime, Utc};

use golion_domain::asset::bess::limits::BessLimits;
use golion_domain::temporal::series::TimeStampedUtc;
use good_lp::{ProblemVariables, Variable, variable};

// region: Bess Variables
/// Bess Variables container with time information
#[derive(Debug)]
pub struct BessVariables {
    pub start_at: DateTime<Utc>,
    pub input_power: Variable,
    pub output_power: Variable,
    pub soc: Variable,
}
// Implement TimeStampedUtc to enable creation
// of TimeSeries<BessVariables> out of Vec<BessVariables>.
impl TimeStampedUtc for BessVariables {
    fn start_at(&self) -> &DateTime<Utc> {
        &self.start_at
    }
}

// endregion: Bess Variables

// region: BessVariableCreator
pub trait BessVariableCreator {
    fn create_variables_at(
        &self,
        dt: &DateTime<Utc>,
        variable_generator: &mut ProblemVariables,
    ) -> Result<BessVariables>;
}

impl BessVariableCreator for BessLimits {
    fn create_variables_at(
        &self,
        dt: &DateTime<Utc>,
        variable_generator: &mut ProblemVariables,
    ) -> Result<BessVariables> {
        let avail_point = self.availability.at(dt)?;
        Ok(BessVariables {
            start_at: *dt,
            input_power: variable_generator
                .add(variable().min(0.0).max(avail_point.max_charge_power)),
            output_power: variable_generator
                .add(variable().min(0.0).max(avail_point.max_discharge_power)),
            soc: variable_generator.add(
                variable()
                    .min(
                        self.soc_range
                            .min_soc
                            .into_energy_kwh(&avail_point.max_usable_energy),
                    )
                    .max(
                        self.soc_range
                            .max_soc
                            .into_energy_kwh(&avail_point.max_usable_energy),
                    ),
            ),
        })
    }
}
// endregion: BessVariableCreator
