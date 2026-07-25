use super::availability::AvailabilityConstraints;
use crate::support::power_to_energy;
use chrono::{DateTime, Utc};

use crate::physical::variables::{BessVariables, VariableCreation};
use golion_domain::temporal::step::MinuteGranularity;
use golion_domain::units::power::KiloWattHour;
use golion_domain::{asset::bess::limits::BessLimits, temporal::series::TimeSeries};
use good_lp::{Constraint, Expression, ProblemVariables, constraint};
// region: Battery Definition
pub struct Battery<A: AvailabilityConstraints> {
    availability: TimeSeries<A>,
    initial_soc: KiloWattHour,
    granularity: MinuteGranularity,
    variables: BessVariables,
    constraints: Vec<Constraint>,
}

impl<A: AvailabilityConstraints> Battery<A> {
    pub fn new(
        time_index: &[DateTime<Utc>],
        vars: &mut ProblemVariables,
        availability: TimeSeries<A>,
        initial_soc: KiloWattHour,
        granularity: MinuteGranularity,
        limits: BessLimits,
    ) -> Result<Self, golion_domain::Error> {
        let time_index_length = time_index.len();
        // Create battery physical variables.
        let variables = limits.create_variables(vars, time_index_length);

        // Initialize battery physical constraints container.
        let mut constraints: Vec<Constraint> = Vec::with_capacity(4 * time_index_length);

        // Loop over time index ,create variables with limits applied ,
        // update physical exchange expression and soc defining constraints
        // and build availability constraints in same loop for efficiency.
        for (i, dt) in time_index.iter().enumerate() {
            let soc_var = variables.soc[i];
            let input_power_var = variables.input_power[i];
            let output_power_var = variables.output_power[i];

            // Create soc transition constraints.
            let prev_soc: Expression = match i {
                0 => initial_soc.0.into(),
                _ => variables.soc[i - 1].into(),
            };
            constraints.push(constraint!(
                soc_var
                    == prev_soc
                        + power_to_energy(
                            input_power_var - output_power_var,
                            granularity.duration()
                        )
            ));
            // Create availability constraints.
            let avail_point = availability.at(dt)?;

            constraints.push(avail_point.charge_power_constraint(input_power_var));
            constraints.push(avail_point.discharge_power_constraint(output_power_var));
            constraints.push(avail_point.energy_available_at_t(soc_var));
        }

        Ok(Self { availability, initial_soc, granularity, variables, constraints })
    }
}
// endregion: Battery Definition

#[cfg(test)]
mod tests {
    use super::Battery;
    use chrono::{DateTime, Duration, Utc};
    use golion_domain::asset::bess::availability::Availability;
    use golion_domain::asset::bess::limits::BessLimits;
    use golion_domain::temporal::grid::RegularTimeGrid;
    use golion_domain::temporal::series::TimeSeries;
    use golion_domain::temporal::step::MinuteGranularity;
    use golion_domain::units::power::{KiloWatt, KiloWattHour};
    use good_lp::ProblemVariables;

    #[test]
    fn build_battery_over_four_slots() {
        // 4 slots at 15-minute granularity.
        let granularity = MinuteGranularity::try_from(Duration::minutes(15)).unwrap();
        let start_at = Utc::now();
        let time_index: Vec<DateTime<Utc>> =
            (0..4).map(|i| start_at + *granularity.duration() * i).collect();

        // One availability point per slot (Availability is Copy).
        let grid =
            RegularTimeGrid::try_new(start_at, granularity, time_index.len()).unwrap();
        let data = vec![
            Availability {
                max_charge_power: KiloWatt(50.0),
                max_discharge_power: KiloWatt(50.0),
                max_usable_energy: KiloWattHour(100.0),
            };
            time_index.len()
        ];
        let availability = TimeSeries { grid, data };

        let mut vars = ProblemVariables::new();
        let limits = BessLimits {
            max_output_power: KiloWatt(50.0),
            max_input_power: KiloWatt(50.0),
            min_soc: KiloWattHour(0.0),
            max_soc: KiloWattHour(100.0),
        };

        let battery = Battery::new(
            &time_index,
            &mut vars,
            availability,
            KiloWattHour(20.0),
            granularity,
            limits,
        )
        .expect("battery construction should succeed");

        // 3 physical variables per slot.
        assert_eq!(battery.variables.input_power.len(), 4);
        assert_eq!(battery.variables.output_power.len(), 4);
        assert_eq!(battery.variables.soc.len(), 4);
        // 1 soc-transition + 3 availability constraints per slot.
        assert_eq!(battery.constraints.len(), 4 * 4);
    }
}
