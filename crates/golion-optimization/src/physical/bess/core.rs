use super::soc::transition_constraint;
use crate::Result;
use crate::physical::variables::{BessVariableCreator, BessVariables};
use chrono::{DateTime, Utc};
use golion_domain::temporal::series::TimeSeries;
use golion_domain::temporal::step::MinuteGranularity;
use golion_domain::units::efficiency::Efficiency;
use golion_domain::units::power::KiloWattHour;
use good_lp::{Constraint, Expression, ProblemVariables};
// region: Battery Definition
pub struct Battery {
    initial_soc: KiloWattHour,
    granularity: MinuteGranularity,
    variable_store: TimeSeries<BessVariables>,
    constraints: Vec<Constraint>,
}

impl Battery {
    pub fn new<B: BessVariableCreator>(
        time_index: &[DateTime<Utc>],
        vars: &mut ProblemVariables,
        charge_efficiency: &Efficiency,
        discharge_efficiency: &Efficiency,
        initial_soc: KiloWattHour,
        granularity: MinuteGranularity,
        limits: B,
    ) -> Result<Self> {
        let time_index_length = time_index.len();
        // Initialize battery physical variables and constraints containers.
        let mut constraints: Vec<Constraint> = Vec::with_capacity(time_index_length);
        let mut variable_vec: Vec<BessVariables> = Vec::with_capacity(time_index_length);
        // Loop over time index ,create variables with their respective limits
        // and generate defining soc constraints.
        for (i, dt) in time_index.iter().enumerate() {
            // Create battery physical variables.
            let variables_at = limits.create_variables_at(dt, vars)?;

            // Create soc transition constraints.
            let prev_soc: Expression = match i {
                0 => initial_soc.0.into(),
                _ => variable_vec[i - 1].soc.into(),
            };
            constraints.push(transition_constraint(
                &variables_at,
                prev_soc,
                &granularity,
                charge_efficiency,
                discharge_efficiency,
            ));
            variable_vec.push(variables_at);
        }
        let variable_store: TimeSeries<BessVariables> = variable_vec.try_into()?;

        Ok(Self { initial_soc, granularity, variable_store, constraints })
    }
}
// endregion: Battery Definition

#[cfg(test)]
mod tests {
    use super::Battery;
    use chrono::{DateTime, Duration, Utc};
    use golion_domain::asset::bess::availability::Availability;
    use golion_domain::asset::bess::limits::{BessLimits, SocRange};
    use golion_domain::temporal::grid::RegularTimeGrid;
    use golion_domain::temporal::series::TimeSeries;
    use golion_domain::temporal::step::MinuteGranularity;
    use golion_domain::units::efficiency::Efficiency;
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
            soc_range: SocRange {
                min_soc: 0.0.try_into().unwrap(),
                max_soc: 1.0.try_into().unwrap(),
            },
            availability,
        };
        let charge_efficiency = Efficiency::try_from(0.95).unwrap();
        let discharge_efficiency = Efficiency::try_from(0.95).unwrap();

        let battery = Battery::new(
            &time_index,
            &mut vars,
            &charge_efficiency,
            &discharge_efficiency,
            KiloWattHour(20.0),
            granularity,
            limits,
        )
        .expect("battery construction should succeed");

        // One physical-variable triple per slot.
        assert_eq!(battery.variable_store.data.len(), 4);
        // One soc-transition constraint per slot.
        assert_eq!(battery.constraints.len(), 4);
    }
}
