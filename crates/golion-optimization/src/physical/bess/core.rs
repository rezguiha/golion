use crate::Result;
use crate::physical::bess::soc::transition_constraint;
use crate::physical::variables::BessVariables;
use golion_domain::asset::bess::availability::Availability;
use golion_domain::asset::bess::limits::SocRange;
use golion_domain::asset::bess::{
    efficiency::BessPowerEfficiencies, specification::BessSpecifications,
};
use golion_domain::temporal::series::TimeSeries;
use golion_domain::temporal::step::MinuteStep;
use golion_domain::units::power::{KiloWatt, KiloWattHour};
use good_lp::{Constraint, Expression, ProblemVariables, Variable, constraint, variable};
use jiff::Timestamp;
// region: BessVariableCreator
pub trait BessVariableCreator {
    fn efficiencies(&self) -> &BessPowerEfficiencies;
    fn soc_range(&self) -> &SocRange;
    fn availability(&self) -> &TimeSeries<Availability>;
    // Creates variables at a particular timestamp t.
    fn create_variables_at(
        &self,
        dt: &Timestamp,
        avail_point: &Availability,
        variable_generator: &mut ProblemVariables,
    ) -> Result<BessVariables> {
        BessVariables::try_new(dt, avail_point, self.soc_range(), variable_generator)
    }
    /// Sets exclusivity constraints between input power and output power.
    /// This is necessary to be able to apply the right efficiency to the active power
    /// of the battery during charge and during discharge.
    fn exclusivity_constraint(
        vars: &mut ProblemVariables,
        input_power: &Variable,
        output_power: &Variable,
        max_charge_power: &KiloWatt,
        max_discharge_power: &KiloWatt,
    ) -> [Constraint; 2] {
        let exclusivity_binary = vars.add(variable().binary());
        let big_m = max_charge_power.0 + max_discharge_power.0;
        [
            constraint!(*input_power <= exclusivity_binary * big_m),
            constraint!(*output_power <= (1 - exclusivity_binary) * big_m),
        ]
    }
    fn create_variables_and_minimal_constraints(
        &self,
        time_index: &[Timestamp],
        vars: &mut ProblemVariables,
        initial_soc: KiloWattHour,
        step: MinuteStep,
    ) -> crate::Result<(TimeSeries<BessVariables>, Vec<Constraint>)> {
        let time_index_length = time_index.len();
        // Initialize battery physical variables and constraints containers.
        let mut constraints: Vec<Constraint> = Vec::with_capacity(3 * time_index_length);
        let mut variable_vec: Vec<BessVariables> = Vec::with_capacity(time_index_length);
        // Loop over time index ,create variables with their respective limits
        // and generate defining soc constraints.
        for (i, dt) in time_index.iter().enumerate() {
            // Create battery physical variables.
            let avail_point = self.availability().at(dt)?;
            let variables_at = self.create_variables_at(dt, avail_point, vars)?;
            // Set exclusivity between active input power and output power.
            constraints.extend(Self::exclusivity_constraint(
                vars,
                &variables_at.input_power,
                &variables_at.output_power,
                &avail_point.max_charge_power,
                &avail_point.max_discharge_power,
            ));
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

// region: Battery Definition
#[derive(Debug)]
pub struct Battery {
    pub(crate) initial_soc: KiloWattHour,
    pub(crate) step: MinuteStep,
    pub(crate) variable_store: TimeSeries<BessVariables>,
    pub(crate) constraints: Vec<Constraint>,
}

impl Battery {
    pub(crate) fn new(
        time_index: &[Timestamp],
        vars: &mut ProblemVariables,
        specifications: &impl BessVariableCreator,
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

// region: Tests
#[cfg(test)]
mod tests {
    use super::Battery;
    use golion_domain::asset::bess::availability::Availability;
    use golion_domain::asset::bess::efficiency::BessPowerEfficiencies;
    use golion_domain::asset::bess::limits::{BessLimits, SocRange};
    use golion_domain::asset::bess::specification::BessSpecifications;
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
        let availability: Vec<Availability> = time_index
            .iter()
            .map(|dt| Availability {
                start_at: *dt,
                max_charge_power: KiloWatt(50.0),
                max_discharge_power: KiloWatt(50.0),
                max_usable_energy: KiloWattHour(100.0),
            })
            .collect();
        let mut vars = ProblemVariables::new();
        let limits = BessLimits {
            soc_range: SocRange::try_new(
                0.0.try_into().unwrap(),
                1.0.try_into().unwrap(),
            )
            .unwrap(),
            availability: availability.try_into().unwrap(),
        };
        let efficiencies = BessPowerEfficiencies {
            charge_efficiency: Efficiency::try_from(0.95).unwrap(),
            discharge_efficiency: Efficiency::try_from(0.95).unwrap(),
        };
        let specifications = BessSpecifications { limits, efficiencies };

        let battery = Battery::new(
            &time_index,
            &mut vars,
            &specifications,
            KiloWattHour(20.0),
            step,
        )
        .expect("battery construction should succeed");

        // One physical-variable triple per slot.
        assert_eq!(battery.variable_store.data().len(), 4);
        // One soc-transition constraint per slot.
        assert_eq!(battery.constraints.len(), 12);
    }
}
// endregion: Tests
