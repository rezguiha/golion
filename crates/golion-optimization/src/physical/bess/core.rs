use super::availability::Availability;
use crate::support::power_to_energy;
use chrono::{DateTime, Utc};

use golion_common::temporal::series::TimeSeries;
use golion_common::temporal::step::MinuteGranularity;
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
    availability: TimeSeries<Availability>,
    initial_soc: KiloWattHour,
    granularity: MinuteGranularity,
    input_power: Box<[Variable]>,
    output_power: Box<[Variable]>,
    soc: Box<[Variable]>,
    constraints: Vec<Constraint>,
}

impl Battery {
    pub fn new(
        time_index: &[DateTime<Utc>],
        vars: &mut ProblemVariables,
        availability: TimeSeries<Availability>,
        initial_soc: KiloWattHour,
        granularity: MinuteGranularity,
        limits: BessLimits,
    ) -> Result<Self, golion_common::Error> {
        let time_index_length = time_index.len();
        // Initialize battery variables containers.
        let mut input_power: Vec<Variable> = Vec::with_capacity(time_index_length);
        let mut output_power: Vec<Variable> = Vec::with_capacity(time_index_length);
        let mut soc: Vec<Variable> = Vec::with_capacity(time_index_length);
        // Initialize battery physical constraints container.
        let mut constraints: Vec<Constraint> = Vec::with_capacity(4 * time_index_length);

        // Loop over time index ,create variables with limits applied ,
        // update physical exchange expression and soc defining constraints
        // and build availability constraints in same loop for efficiency.
        for (i, dt) in time_index.iter().enumerate() {
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

            // Create soc transition constraints.
            let prev_soc: Expression = match i {
                0 => initial_soc.0.into(),
                _ => soc[i - 1].into(),
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

        Ok(Self {
            availability,
            initial_soc,
            granularity,
            input_power: input_power.into_boxed_slice(),
            output_power: output_power.into_boxed_slice(),
            soc: soc.into_boxed_slice(),
            constraints,
        })
    }
}
// endregion: Battery Definition

#[cfg(test)]
mod tests {
    use super::{Battery, BessLimits};
    use crate::physical::bess::availability::Availability;
    use chrono::{DateTime, Duration, Utc};
    use golion_common::temporal::grid::RegularTimeGrid;
    use golion_common::temporal::series::TimeSeries;
    use golion_common::temporal::step::MinuteGranularity;
    use golion_common::units::power::{KiloWatt, KiloWattHour};
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
        assert_eq!(battery.input_power.len(), 4);
        assert_eq!(battery.output_power.len(), 4);
        assert_eq!(battery.soc.len(), 4);
        // 1 soc-transition + 3 availability constraints per slot.
        assert_eq!(battery.constraints.len(), 4 * 4);
    }
}
