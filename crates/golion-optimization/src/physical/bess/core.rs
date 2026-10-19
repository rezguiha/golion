use crate::Result;
use crate::model::Horizon;
use crate::physical::ActivationEnergy;
use crate::physical::bess::soc::transition_constraint;
use crate::physical::bess::throughput::create_cycling_constraints;
use crate::physical::variables::BessVariables;
use golion_domain::asset::bess::availability::Availability;
use golion_domain::asset::bess::efficiency::BessPowerEfficiencies;
use golion_domain::asset::bess::soc::SocBounds;
use golion_domain::asset::bess::specification::BessSpecifications;
use golion_domain::asset::state::BessState;
use golion_domain::temporal::series::TimeSeries;
use golion_domain::temporal::step::MinuteStep;
use golion_domain::units::power::{KiloWatt, KiloWattHour};
use good_lp::{
    Constraint, Expression, IntoAffineExpression, ProblemVariables, constraint,
};
use jiff::Timestamp;
#[derive(Debug)]
pub struct BessInitialState {
    remaining_upward_activation: KiloWattHour,
    remaining_downward_activation: KiloWattHour,
    soc: KiloWattHour,
}
impl BessInitialState {
    /// Adds dispatch to initial soc and make sure to stay inside the soc_min and
    /// soc_max range so we do not trigger infeasabilities that needlessly block
    /// the optimization process as asset will not fulfill full dispatch signal
    /// and will stick to what is feasible. We also cap the considered ancillary
    /// repartition to start between soc_min and soc_max for the same reason.
    fn try_new(
        current_state: &BessState,
        initial_soc: KiloWattHour,
        horizon: &Horizon,
        initial_soc_bound: &SocBounds,
        efficiencies: &BessPowerEfficiencies,
    ) -> crate::Result<Self> {
        let time_to_start = horizon.duration_to_grid_start()?;
        let energy = |power: KiloWatt| power.to_kwh(time_to_start).0;
        // Compute delta soc with correct efficiency applied depending on sign
        // of dispatch.
        let delta_soc = match energy(current_state.dispatch) {
            d if d > 0.0 => d * efficiencies.charge_efficiency.value(),
            d => d / efficiencies.discharge_efficiency.value(),
        };
        // Compute clipped initial soc for wholesale only scenario.
        let soc = (initial_soc.0 + delta_soc)
            .clamp(initial_soc_bound.min.0, initial_soc_bound.max.0);
        // Upward activation discharges: capped by the energy left above soc_min.
        // We make sure to use the same conservative projection used in soc
        // reservation constraints for ancillary commitments.
        let upward = energy(current_state.ancillary_upward).min(
            (soc - initial_soc_bound.min.0) * efficiencies.discharge_efficiency.value(),
        );
        // Downward activation charges: capped by the room left below soc_max.
        // We make sure to use the same conservative projection used in soc
        // reservation constraints for ancillary commitments.
        let downward = energy(current_state.ancillary_downward).min(
            (initial_soc_bound.max.0 - soc) * efficiencies.discharge_efficiency.value(),
        );

        Ok(Self {
            remaining_upward_activation: upward.into(),
            remaining_downward_activation: downward.into(),
            soc: soc.into(),
        })
    }
}

// region: Battery Definition
#[derive(Debug)]
pub struct Battery {
    initial_state: BessInitialState,
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
        horizon: &Horizon,
        vars: &mut ProblemVariables,
        specifications: BessSpecifications,
        initial_soc: KiloWattHour,
        step: MinuteStep,
        current_state: BessState,
    ) -> Result<Self> {
        let grid_start = &horizon.grid().start();
        let soc_bounds = specifications.soc_bounds()?;
        let initial_soc_bound = soc_bounds.at(grid_start)?;
        let initial_state = BessInitialState::try_new(
            &current_state,
            initial_soc,
            horizon,
            initial_soc_bound,
            specifications.efficiencies(),
        )?;

        let (variable_store, constraints) =
            Self::create_variables_and_minimal_constraints(
                &specifications,
                horizon.timestamps(),
                vars,
                &soc_bounds,
                initial_state.soc,
                step,
            )?;

        Ok(Self {
            initial_state,
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
        let mut upward =
            self.initial_state.remaining_upward_activation.0.into_expression();
        let mut downward =
            self.initial_state.remaining_downward_activation.0.into_expression();
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
    use crate::model::Horizon;

    use super::Battery;
    use golion_domain::asset::bess::availability::Availability;
    use golion_domain::asset::bess::efficiency::BessPowerEfficiencies;
    use golion_domain::asset::bess::soc::SocRange;
    use golion_domain::asset::bess::specification::BessSpecifications;
    use golion_domain::asset::bess::throughput::{
        BidirectionalThroughput, Cycling, ThroughputHistory,
    };
    use golion_domain::asset::state::BessState;
    use golion_domain::temporal::grid::RegularTimeGrid;
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
        let grid = RegularTimeGrid::try_new(start_at, step, 4).unwrap();
        let horizon = Horizon::new(&start_at, &grid);
        let rated_charge_power = KiloWatt(50.0);
        let rated_discharge_power = KiloWatt(50.0);
        let rated_energy = KiloWattHour(100.0);
        let availability: Vec<Availability> = horizon
            .timestamps()
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
        let initial_soc = KiloWattHour(20.0);
        let current_state = BessState {
            ancillary_downward: 0.0.into(),
            ancillary_upward: 0.0.into(),
            dispatch: 0.0.into(),
            start_at: start_at - *step.duration(),
            soc: initial_soc,
        };

        let battery = Battery::new(
            &horizon,
            &mut vars,
            specifications,
            initial_soc,
            step,
            current_state,
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
