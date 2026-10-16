use crate::Result;
use crate::physical::ActivationEnergy;
use crate::physical::bess::soc::transition_constraint;
use crate::physical::bess::throughput::create_cycling_constraints;
use crate::physical::variables::BessVariables;
use golion_domain::asset::bess::availability::Availability;
use golion_domain::asset::bess::soc::SocBounds;
use golion_domain::asset::bess::specification::BessSpecifications;
use golion_domain::temporal::series::TimeSeries;
use golion_domain::temporal::step::MinuteStep;
use golion_domain::units::power::{KiloWatt, KiloWattHour};
use good_lp::{
    Constraint, Expression, IntoAffineExpression, ProblemVariables, constraint,
};
use jiff::Timestamp;

// region: Battery Definition
#[derive(Debug)]
pub struct Battery {
    initial_soc: KiloWattHour,
    step: MinuteStep,
    pub(crate) variable_store: TimeSeries<BessVariables>,
    pub(crate) constraints: Vec<Constraint>,
    soc_bounds: TimeSeries<SocBounds>,
    // The specifications are temporarily put in a box
    // as it is accessed in a limited way here.
    // This has been done to handle a lint error about enum
    // variants size mismatch over 200 bytes. This is only
    // present because Gasturbine and Renewable variants are
    // not implemented yet and do not contain their specifications
    // that will be defined later on. When that is done the sizes
    // will be similar and we should put this back to the stack.
    specifications: Box<BessSpecifications>,
}

impl Battery {
    // Creates variables at a particular timestamp t.
    fn create_variables_at(
        dt: &Timestamp,
        avail_point: &Availability,
        soc_bounds: &SocBounds,
        variable_generator: &mut ProblemVariables,
    ) -> Result<BessVariables> {
        BessVariables::try_new(dt, avail_point, soc_bounds, variable_generator)
    }
    fn create_variables_and_minimal_constraints(
        specifications: &BessSpecifications,
        time_index: &[Timestamp],
        vars: &mut ProblemVariables,
        soc_bounds: &TimeSeries<SocBounds>,
        initial_soc: KiloWattHour,
        step: MinuteStep,
    ) -> crate::Result<(TimeSeries<BessVariables>, Vec<Constraint>)> {
        let time_index_length = time_index.len();
        // Initialize battery physical variables and constraints containers.
        let mut constraints: Vec<Constraint> = Vec::with_capacity(5 * time_index_length);
        let mut variable_vec: Vec<BessVariables> = Vec::with_capacity(time_index_length);
        // Loop over time index ,create variables with their respective limits
        // and generate defining soc constraints.
        for (i, dt) in time_index.iter().enumerate() {
            // Create battery physical variables.
            let avail_point = specifications.availability.at(dt)?;
            let variables_at =
                Self::create_variables_at(dt, avail_point, soc_bounds.at(dt)?, vars)?;
            // Set exclusivity between active input power and output power.
            // This is necessary to be able to apply the right efficiency to the active power
            // of the battery during charge and during discharge.
            constraints.extend(variables_at.exclusivity_constraint(
                vars,
                &avail_point.max_charge_power,
                &avail_point.max_discharge_power,
            ));
            // Leave enough power room around the planned dispatch to fully
            // activate the reserved ancillary power.
            constraints.extend(variables_at.headroom_constraint(
                &avail_point.max_charge_power,
                &avail_point.max_discharge_power,
            ));
            // Create soc transition constraint.
            let previous_soc: Expression = match i {
                0 => initial_soc.0.into(),
                _ => variable_vec[i - 1].soc.into_expression(),
            };
            constraints.push(transition_constraint(
                &variables_at,
                previous_soc,
                &step,
                &specifications.efficiencies.charge_efficiency,
                &specifications.efficiencies.discharge_efficiency,
            ));
            variable_vec.push(variables_at);
        }
        let variable_store: TimeSeries<BessVariables> = variable_vec.try_into()?;

        Ok((variable_store, constraints))
    }
}
impl Battery {
    pub(crate) fn new(
        time_index: &[Timestamp],
        vars: &mut ProblemVariables,
        specifications: BessSpecifications,
        initial_soc: KiloWattHour,
        step: MinuteStep,
    ) -> Result<Self> {
        let soc_bounds = specifications.soc_bounds()?;
        let (variable_store, constraints) =
            Self::create_variables_and_minimal_constraints(
                &specifications,
                time_index,
                vars,
                &soc_bounds,
                initial_soc,
                step,
            )?;
        Ok(Self {
            initial_soc,
            step,
            variable_store,
            constraints,
            specifications: specifications.into(),
            soc_bounds,
        })
    }
    /// State of charge reservation constraints along an activation path: at every
    /// step, the planned state of charge must hold, within its bounds, the energy
    /// drawn since the start of the activation. The planned state of charge already
    /// includes every planned wholesale flow up to that step, so the activation is
    /// projected on top of the planned dispatch.
    ///
    /// We are using a conservative approximation here with the usage of 1/discharge_efficiency
    /// times ancillary commitments. This enables us to avoid using binaries for each
    /// timestep and step in activation path which is computationaly very heavy.
    pub(crate) fn soc_reservation_constraints(
        &self,
        activation_path: &[ActivationEnergy],
    ) -> Result<Vec<Constraint>> {
        // Largest state of charge variation per kWh activated, whichever way the
        // battery delivers the activation, as charge efficiency is at most 1.
        let factor =
            1.0_f64 / self.specifications.efficiencies.discharge_efficiency.value();
        // Activation energy drawn since the start of the activation.
        let mut upward = 0.0.into_expression();
        let mut downward = 0.0.into_expression();
        let mut constraints = Vec::with_capacity(2 * activation_path.len());
        for energy in activation_path {
            upward += &energy.upward;
            downward += &energy.downward;
            let soc = self.variable_store.at(&energy.start_at)?.soc;
            let soc_bounds = self.soc_bounds.at(&energy.start_at)?;
            let mut lowest_soc = soc.into_expression();
            lowest_soc.add_mul(-factor, &upward);
            let mut highest_soc = soc.into_expression();
            highest_soc.add_mul(factor, &downward);
            constraints.push(constraint!(lowest_soc >= soc_bounds.min.0));
            constraints.push(constraint!(highest_soc <= soc_bounds.max.0));
        }
        Ok(constraints)
    }

    pub(crate) fn cycling_constraints(
        &self,
        expected_activation_series: &TimeSeries<ActivationEnergy>,
    ) -> crate::Result<Vec<Constraint>> {
        create_cycling_constraints(
            &self.specifications,
            &self.variable_store,
            expected_activation_series,
        )
    }
}

impl Battery {
    pub fn rated_input_power(&self) -> &KiloWatt {
        &self.specifications.rated_charge_power
    }
    pub fn rated_output_power(&self) -> &KiloWatt {
        &self.specifications.rated_discharge_power
    }
    pub fn specifications(&self) -> &BessSpecifications {
        &self.specifications
    }
}
// endregion: Battery Definition

// region: Tests
#[cfg(test)]
mod tests {
    use super::Battery;
    use golion_domain::asset::bess::availability::Availability;
    use golion_domain::asset::bess::efficiency::BessPowerEfficiencies;
    use golion_domain::asset::bess::soc::SocRange;
    use golion_domain::asset::bess::specification::BessSpecifications;
    use golion_domain::asset::bess::throughput::{
        BidirectionalThroughput, Cycling, ThroughputHistory,
    };
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
        let rated_charge_power = KiloWatt(50.0);
        let rated_discharge_power = KiloWatt(50.0);
        let rated_energy = KiloWattHour(100.0);
        let availability: Vec<Availability> = time_index
            .iter()
            .map(|dt| Availability {
                start_at: *dt,
                max_charge_power: rated_charge_power,
                max_discharge_power: rated_discharge_power,
                max_usable_energy: rated_energy,
            })
            .collect();
        let mut vars = ProblemVariables::new();
        let soc_range =
            SocRange::try_new(0.0.try_into().unwrap(), 1.0.try_into().unwrap()).unwrap();
        let efficiencies = BessPowerEfficiencies {
            charge_efficiency: Efficiency::try_from(0.95).unwrap(),
            discharge_efficiency: Efficiency::try_from(0.95).unwrap(),
        };
        let history: Vec<BidirectionalThroughput> = (1..=96)
            .rev()
            .map(|i| BidirectionalThroughput {
                start_at: start_at - *step.duration() * i,
                charged_energy: KiloWattHour(0.0),
                discharged_energy: KiloWattHour(0.0),
            })
            .collect();
        let cycling = Cycling {
            history: ThroughputHistory::Bidirectional(history.try_into().unwrap()),
            max_24h_avg_cycles: 1.0,
            penalty: 0.0,
        };
        let specifications = BessSpecifications {
            soc_range,
            availability: availability.try_into().unwrap(),
            cycling,
            efficiencies,
            rated_charge_power,
            rated_discharge_power,
            rated_energy,
        };

        let battery = Battery::new(
            &time_index,
            &mut vars,
            specifications,
            KiloWattHour(20.0),
            step,
        )
        .expect("battery construction should succeed");

        // One set of physical variables per slot.
        assert_eq!(battery.variable_store.data().len(), 4);
        // One soc-transition constraint per slot.
        // Two Exclusivity of active power constraints per slot.
        // Two ancillary power headroom constraints per slot.
        assert_eq!(battery.constraints.len(), 20);
    }
}
// endregion: Tests
