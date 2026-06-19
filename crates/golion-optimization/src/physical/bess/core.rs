use super::availability::Availability;
use crate::error::SeriesError;
use chrono::{DateTime, Duration, Utc};
use golion_common::units::power::{KiloWatt, KiloWattHour};
use good_lp::{Constraint, Expression, ProblemVariables, Variable, constraint, variable};
// region: Battery Limits
pub struct BessLimits {
    pub max_output_power: KiloWatt,
    pub max_input_power: KiloWatt,
    pub min_soc: KiloWattHour,
    pub max_soc: KiloWattHour,
}
// endregion: Battery Limits
// region: Battery Definition
pub struct Battery {
    availability: Vec<Availability>,
    initial_soc: KiloWattHour,
    granularity: Duration,
    input_power: Vec<Variable>,
    output_power: Vec<Variable>,
    soc: Vec<Variable>,
    constraints: Vec<Constraint>,
}

impl Battery {
    pub fn new(
        time_index: &[DateTime<Utc>],
        vars: &mut ProblemVariables,
        availability: Vec<Availability>,
        initial_soc: KiloWattHour,
        granularity: Duration,
        limits: BessLimits,
    ) -> Result<Self, SeriesError> {
        // Check availability length matches time index length.
        let time_index_length = time_index.len();
        let availability_length = availability.len();
        if availability_length != time_index_length {
            return Err(SeriesError::MismatchedLength {
                entity: "Availability".to_string(),
                length: availability_length,
                reference_length: time_index_length,
            });
        }
        // Initialize battery variables containers.
        let mut input_power: Vec<Variable> = Vec::with_capacity(time_index_length);
        let mut output_power: Vec<Variable> = Vec::with_capacity(time_index_length);
        let mut soc: Vec<Variable> = Vec::with_capacity(time_index_length);
        // Initialize battery physical constraints container.
        let mut constraints: Vec<Constraint> = Vec::with_capacity(4 * time_index_length);
        let granularity_hours = granularity.as_seconds_f64() / 3600.0;
        let mut physical_exchange: Expression = initial_soc.0.into();
        // Loop over time index ,create variables with limits applied ,
        // update physical exchange expression and soc defining constraints
        // and build availability constraints in same loop for efficiency.
        for (_, avail_point) in time_index.iter().zip(availability.iter()) {
            // Create battery physical variables.
            let input_power_var =
                vars.add(variable().min(0.0).max(limits.max_input_power.0));
            let output_power_var =
                vars.add(variable().min(0.0).max(limits.max_output_power.0));
            let soc_var =
                vars.add(variable().min(limits.min_soc.0).max(limits.max_soc.0));
            input_power.push(input_power_var);
            output_power.push(output_power_var);
            soc.push(soc_var);

            // Create soc definition constraints
            physical_exchange += (input_power_var - output_power_var) * granularity_hours;
            constraints.push(constraint!(soc_var == physical_exchange.clone()));
            // Create availability constraints.

            constraints.push(avail_point.charge_power_constraint(input_power_var));
            constraints.push(avail_point.discharge_power_constraint(output_power_var));
            constraints.push(avail_point.energy_available_at_t(soc_var));
        }

        Ok(Self {
            availability,
            initial_soc,
            granularity,
            input_power,
            output_power,
            soc,
            constraints,
        })
    }
}
// endregion: Battery Definition
