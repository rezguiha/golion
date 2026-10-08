use super::support::{
    aggregate_market_bids, build_penalization_expression, build_penalization_variables,
};
use crate::{
    market::{core::Market, variables::BidVariables},
    model::BuildEnv,
    physical::{ActivationEnergy, ActivationPath, PhysicalStore},
};
use golion_domain::{
    market::{commitment::PowerCommitment, market_type::AncillaryMarketType},
    problem::definition::ReserveDefinition,
    solution::{MarketSolution, ReserveSolution, SolutionWithRevenue},
    temporal::series::TimeSeries,
};
use good_lp::{
    Constraint, Expression, IntoAffineExpression, ProblemVariables, constraint, variable,
};
use jiff::{SignedDuration, Timestamp};
use std::collections::HashMap;

use uuid::Uuid;

/// Perimeter for the ancillary service containing
/// assets certified together for it.
#[derive(Debug)]
pub struct ReservePerimeter {
    /// Id of the perimeter definition.
    id: Uuid,
    /// Ancillary Service
    market: Market,
    /// Perimeter level aggregated bid variables of its market.
    bid_store: TimeSeries<BidVariables>,
    /// Repartition variables/expressions per asset of ancillary commitments
    /// and bidding over each timestamp in time index
    repartition: HashMap<Uuid, TimeSeries<BidVariables>>,
    /// Container for constraints for repartition.
    constraints: Vec<Constraint>,
    /// Reserve level penalization in order to avoid violations.
    /// This represents the imbalance.
    penalization_store: TimeSeries<BidVariables>,
    /// Reserve level revenue expression.
    revenue: Expression,
    /// Reserve level penalty expression.
    penalty: Expression,
    /// Number of grid steps a full activation must be sustained for.
    activation_steps: usize,
}

impl ReservePerimeter {
    /// Builds the perimeter ancillary market and the repartition of its
    /// bids and commitments over its assets.
    pub(crate) fn try_new(
        definition: &ReserveDefinition,
        env: &BuildEnv<'_>,
        vars: &mut ProblemVariables,
    ) -> crate::Result<Self> {
        let horizon = env.horizon();
        let market = Market::try_new(definition.market(), env, vars)?;
        let bid_store = aggregate_market_bids(
            horizon.timestamps(),
            horizon.grid(),
            std::slice::from_ref(&market),
        )?;
        let repartition = definition
            .composition()
            .iter()
            .map(|id| {
                Self::create_asset_repartition_variables(
                    vars,
                    horizon.timestamps(),
                    horizon.grid().step().duration(),
                )
                .map(|series| (*id, series))
            })
            .collect::<crate::Result<HashMap<Uuid, TimeSeries<BidVariables>>>>()?;
        // Build penalization Variables.
        let penalization_store = build_penalization_variables(horizon, vars)?;
        // Build penalization constraints.
        let mut constraints: Vec<Constraint> = Vec::new();
        Self::build_repartition_constraints(
            horizon.timestamps(),
            &mut constraints,
            &bid_store,
            definition.commitments(),
            &penalization_store,
            &repartition,
        )?;
        // Build revenue and penalty expressions
        let revenue = 0.0.into_expression() + market.revenue();
        let penalty =
            build_penalization_expression(&penalization_store, *definition.penalty());

        Ok(Self {
            id: *definition.id(),
            market,
            bid_store,
            repartition,
            constraints,
            penalization_store,
            revenue,
            penalty,
            activation_steps: definition.activation_steps(),
        })
    }
    /// Creates reserve aggregated bidding and commitments
    /// repartition per asset.
    fn create_asset_repartition_variables(
        vars: &mut ProblemVariables,
        time_index: &[Timestamp],
        step: &SignedDuration,
    ) -> crate::Result<TimeSeries<BidVariables>> {
        Ok(time_index
            .iter()
            .map(|dt| {
                BidVariables::new(
                    *dt,
                    vars.add(variable().min(0.0)).into_expression(),
                    vars.add(variable().min(0.0)).into_expression(),
                    step,
                )
            })
            .collect::<Vec<_>>()
            .try_into()?)
    }
    /// Sets the perimeter target (commitments and new bids, minus the part
    /// its assets cannot deliver) equal to the sum of the asset repartition.
    fn build_repartition_constraints(
        time_index: &[Timestamp],
        constraints: &mut Vec<Constraint>,
        bid_store: &TimeSeries<BidVariables>,
        commitments: &TimeSeries<PowerCommitment>,
        penalization_store: &TimeSeries<BidVariables>,
        asset_level_repartition_variables: &HashMap<Uuid, TimeSeries<BidVariables>>,
    ) -> crate::Result<()> {
        for dt in time_index.iter() {
            let bids = bid_store.at(dt)?;
            let commitment = commitments.at(dt)?;
            let penalization = penalization_store.at(dt)?;
            // Reserves are held both ways: commitments and new bids add up per
            // direction and are never netted.
            let mut perimeter_input_power = commitment.input_power.0.into_expression();
            let mut perimeter_output_power = commitment.output_power.0.into_expression();
            perimeter_input_power += bids.input_power();
            perimeter_output_power += bids.output_power();
            // Penalization relaxes what the assets must reserve, covering the
            // part of the perimeter target they cannot deliver.
            perimeter_input_power.add_mul(-1.0, penalization.input_power());
            perimeter_output_power.add_mul(-1.0, penalization.output_power());
            let mut assets_input_power = 0.0.into_expression();
            let mut assets_output_power = 0.0.into_expression();
            for asset_series in asset_level_repartition_variables.values() {
                let asset = asset_series.at(dt)?;
                assets_input_power += asset.input_power();
                assets_output_power += asset.output_power();
            }
            constraints.push(constraint!(perimeter_input_power == assets_input_power));
            constraints.push(constraint!(perimeter_output_power == assets_output_power));
        }
        Ok(())
    }
    /// Moves the perimeter and its market constraints out, leaving them empty.
    pub(crate) fn take_constraints(&mut self) -> impl Iterator<Item = Constraint> {
        std::mem::take(&mut self.constraints)
            .into_iter()
            .chain(self.market.take_constraints())
    }
}
/// Container of all ancillary service perimeters.
#[derive(Debug)]
pub struct AncillaryPerimeter {
    reserve_perimeters: Vec<ReservePerimeter>,
    constraints: Vec<Constraint>,
    revenue: Expression,
    penalty: Expression,
}

impl AncillaryPerimeter {
    pub(crate) fn try_new(
        definitions: &[ReserveDefinition],
        physical_store: &PhysicalStore,
        env: &BuildEnv<'_>,
        vars: &mut ProblemVariables,
    ) -> crate::Result<Self> {
        let reserve_perimeters: Vec<ReservePerimeter> = definitions
            .iter()
            .map(|definition| ReservePerimeter::try_new(definition, env, vars))
            .collect::<crate::Result<_>>()?;
        let mut constraints = Vec::<Constraint>::new();
        Self::physical_reserve_perimeters_constraints(
            &reserve_perimeters,
            env.horizon().timestamps(),
            physical_store,
            &mut constraints,
        )?;
        Self::activation_energy_constraints(
            &reserve_perimeters,
            env.horizon().timestamps(),
            physical_store,
            &mut constraints,
        )?;
        // Compute Overall Revnue.
        let mut revenue = 0.0.into_expression();
        let mut penalty = 0.0.into_expression();
        for reserve in reserve_perimeters.iter() {
            revenue += &reserve.revenue;
            penalty += &reserve.penalty
        }

        Ok(Self { reserve_perimeters, constraints, revenue, penalty })
    }
    /// Links each asset's physical ancillary variables to the sum of its
    /// repartition over every reserve perimeter it belongs to.
    fn physical_reserve_perimeters_constraints(
        reserve_perimiters: &[ReservePerimeter],
        time_index: &[Timestamp],
        physical_store: &PhysicalStore,
        constraints: &mut Vec<Constraint>,
    ) -> crate::Result<()> {
        for (asset_id, asset) in physical_store.iter() {
            // Determine the reserve perimeters in which the asset is present
            let repartitions: Vec<&TimeSeries<BidVariables>> = reserve_perimiters
                .iter()
                .filter_map(|perimeter| perimeter.repartition.get(asset_id))
                .collect();
            for dt in time_index.iter() {
                let mut reserve_input_power = 0.0.into_expression();
                let mut reserve_output_power = 0.0.into_expression();
                let asset_input_power = asset.ancillary_input_power_at(dt)?;
                let asset_output_power = asset.ancillary_output_power_at(dt)?;
                // Aggregate asset repartition on all reserve perimeters it belong to.
                for repartition in &repartitions {
                    let asset_reserve_variables = repartition.at(dt)?;
                    reserve_input_power += asset_reserve_variables.input_power();
                    reserve_output_power += asset_reserve_variables.output_power();
                }
                // Make sure aggregation is equal to the reserved signal on the asset
                // for ancillary services.
                constraints.push(constraint!(asset_input_power == reserve_input_power));
                constraints.push(constraint!(asset_output_power == reserve_output_power));
            }
        }
        Ok(())
    }
    /// Makes each asset able to deliver the energy of a full activation of its
    /// repartitions. For an activation starting at each timestamp, every reserve
    /// draws energy only within its own activation window: the resulting path is
    /// handed to the asset, which sets its energy constraints along it.
    fn activation_energy_constraints(
        reserve_perimiters: &[ReservePerimeter],
        time_index: &[Timestamp],
        physical_store: &PhysicalStore,
        constraints: &mut Vec<Constraint>,
    ) -> crate::Result<()> {
        for (asset_id, asset) in physical_store.iter() {
            // Activation window and repartition of every reserve perimeter
            // in which the asset is present.
            let windows: Vec<(usize, &TimeSeries<BidVariables>)> = reserve_perimiters
                .iter()
                .filter_map(|perimeter| {
                    perimeter
                        .repartition
                        .get(asset_id)
                        .map(|repartition| (perimeter.activation_steps, repartition))
                })
                .collect();
            let Some(longest_window) = windows.iter().map(|(steps, _)| *steps).max()
            else {
                continue;
            };
            for start in 0..time_index.len() {
                // The activation path is cut at the end of the horizon.
                let steps = time_index[start..]
                    .iter()
                    .take(longest_window)
                    .enumerate()
                    .map(|(offset, dt)| {
                        let mut energy = ActivationEnergy {
                            start_at: *dt,
                            upward: 0.0.into_expression(),
                            downward: 0.0.into_expression(),
                        };
                        for (activation_steps, repartition) in &windows {
                            // Each reserve draws energy only within its own
                            // activation window.
                            if offset < *activation_steps {
                                let share = repartition.at(dt)?;
                                energy.upward += share.output_energy();
                                energy.downward += share.input_energy();
                            }
                        }
                        Ok(energy)
                    })
                    .collect::<crate::Result<Vec<_>>>()?;
                constraints.extend(
                    asset.ancillary_energy_constraints(&ActivationPath { steps })?,
                );
            }
        }
        Ok(())
    }
    pub fn reserve_perimeters(&self) -> &[ReservePerimeter] {
        &self.reserve_perimeters
    }
    pub(crate) fn revenue(&self) -> &Expression {
        &self.revenue
    }
    pub(crate) fn penalty(&self) -> &Expression {
        &self.penalty
    }
    /// Moves the ancillary constraints and those of every reserve perimeter
    /// out, leaving them empty.
    pub(crate) fn take_constraints(&mut self) -> impl Iterator<Item = Constraint> {
        std::mem::take(&mut self.constraints).into_iter().chain(
            self.reserve_perimeters
                .iter_mut()
                .flat_map(ReservePerimeter::take_constraints),
        )
    }
}

// region: Solution conversion

impl ReservePerimeter {
    pub(crate) fn to_solution(
        &self,
        solution: &impl good_lp::Solution,
    ) -> crate::Result<ReserveSolution> {
        let revenue = solution.eval(self.market.revenue());
        let market_solution = MarketSolution {
            market_type: AncillaryMarketType::try_from(*self.market.market_type())?,
            solution: SolutionWithRevenue {
                revenue,
                series: self
                    .market
                    .bid_variables()
                    .map(|bid_variables| bid_variables.to_ancillary_bid(solution))
                    .collect(),
                step: *self.market.step(),
            },
        };
        let shortages = self
            .penalization_store
            .data()
            .iter()
            .map(|bid_variables| bid_variables.to_ancillary_bid(solution))
            .collect();
        Ok(ReserveSolution {
            id: self.id,
            solution: market_solution,
            penalty: solution.eval(&self.penalty),
            shortages,
        })
    }
}
// endregion: Solution conversion
