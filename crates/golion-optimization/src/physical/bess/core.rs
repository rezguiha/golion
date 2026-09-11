use crate::Result;
use crate::physical::variables::{BessVariableCreator, BessVariables};
use golion_domain::temporal::series::TimeSeries;
use golion_domain::temporal::step::MinuteStep;
use golion_domain::units::power::KiloWattHour;
use good_lp::{Constraint, ProblemVariables};
use jiff::Timestamp;
// region: Battery Definition
pub struct Battery {
    initial_soc: KiloWattHour,
    step: MinuteStep,
    variable_store: TimeSeries<BessVariables>,
    constraints: Vec<Constraint>,
}

impl Battery {
    pub fn new(
        time_index: &[Timestamp],
        vars: &mut ProblemVariables,
        specifications: impl BessVariableCreator,
        initial_soc: KiloWattHour,
        step: MinuteStep,
    ) -> Result<Self> {
        let (variable_store, constraints) = specifications
            .create_variables_and_minimal_constraints(
                time_index,
                vars,
                initial_soc,
                step,
            )?;
        Ok(Self { initial_soc, step, variable_store, constraints })
    }
}
// endregion: Battery Definition

#[cfg(test)]
mod tests {
    use super::Battery;
    use golion_domain::asset::bess::availability::Availability;
    use golion_domain::asset::bess::efficiency::BessPowerEfficiencies;
    use golion_domain::asset::bess::limits::{BessLimits, SocRange};
    use golion_domain::asset::bess::specification::BessSpecifications;
    use golion_domain::temporal::grid::RegularTimeGrid;
    use golion_domain::temporal::series::TimeSeries;
    use golion_domain::temporal::step::MinuteStep;
    use golion_domain::units::efficiency::Efficiency;
    use golion_domain::units::power::{KiloWatt, KiloWattHour};
    use good_lp::ProblemVariables;
    use jiff::{SignedDuration, Timestamp, Unit};

    #[test]
    fn build_battery_over_four_slots() {
        // 4 slots at 15-minute step.
        let step = MinuteStep::try_from(SignedDuration::from_mins(15)).unwrap();
        let start_at =
            Timestamp::now().round((Unit::Minute, step.duration().as_mins())).unwrap();
        let time_index: Vec<Timestamp> =
            (0..4).map(|i| start_at + *step.duration() * i).collect();

        // One availability point per slot (Availability is Copy).
        let grid = RegularTimeGrid::try_new(start_at, step, time_index.len()).unwrap();
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
        let efficiencies = BessPowerEfficiencies {
            charge_efficiency: Efficiency::try_from(0.95).unwrap(),
            discharge_efficiency: Efficiency::try_from(0.95).unwrap(),
        };
        let specifications = BessSpecifications { limits, efficiencies };

        let battery = Battery::new(
            &time_index,
            &mut vars,
            specifications,
            KiloWattHour(20.0),
            step,
        )
        .expect("battery construction should succeed");

        // One physical-variable triple per slot.
        assert_eq!(battery.variable_store.data.len(), 4);
        // One soc-transition constraint per slot.
        assert_eq!(battery.constraints.len(), 4);
    }
}
