use super::support::{aggregate_market_bids, build_penalization_variables};
use crate::{
    market::{core::Market, variables::BidVariables},
    model::BuildEnv,
    perimeter::support::build_penalization_expression,
    physical::PhysicalStore,
};
use golion_domain::{
    market::{
        bid::WholesaleBid, commitment::PowerCommitment, market_type::WholesaleMarketType,
    },
    problem::definition::BrpDefinition,
    solution::{BrpSolution, MarketSolution, SolutionWithRevenue},
    temporal::series::TimeSeries,
};
use good_lp::{
    Constraint, Expression, IntoAffineExpression, ProblemVariables, constraint,
};
use uuid::Uuid;

// region: Balance Responsible Party Perimeter
#[derive(Debug)]
pub struct BrpPerimeter {
    /// Id of the perimeter definition.
    id: Uuid,
    /// List of wholesale markets to bid on
    markets: Vec<Market>,
    /// Perimeter level aggregated bid variables of its markets. Commitments
    /// are kept out, as they are sunk constants and never decided upon.
    bid_store: TimeSeries<BidVariables>,
    /// Perimeter constraints.
    constraints: Vec<Constraint>,
    /// Perimeter level penalization in order to avoid violations.
    /// This represents the imbalance.
    penalization_store: TimeSeries<BidVariables>,
    /// BRP level revenue expression including penalization
    revenue: Expression,
    /// BRP level penalization expression
    penalty: Expression,
}

impl BrpPerimeter {
    /// Builds the perimeter markets and links the perimeter net position
    /// to its assets.
    pub(crate) fn try_new(
        definition: &BrpDefinition,
        physical_store: &PhysicalStore,
        env: &BuildEnv<'_>,
        vars: &mut ProblemVariables,
    ) -> crate::Result<Self> {
        let horizon = env.horizon();
        // Create the markets of the perimeter.
        let markets: Vec<Market> = definition
            .markets()
            .iter()
            .map(|market_specs| Market::try_new(market_specs, env, vars))
            .collect::<crate::Result<_>>()?;
        // Aggregate market bids at brp perimeter level.
        let bid_store =
            aggregate_market_bids(horizon.timestamps(), horizon.grid(), &markets)?;
        // Create penalization variables
        let penalization_store = build_penalization_variables(horizon, vars)?;
        // Add Repartition Constraints
        let mut constraints =
            Vec::<Constraint>::with_capacity(horizon.timestamps().len());
        Self::build_exclusivity_and_repartition_constraints(
            vars,
            &mut constraints,
            &bid_store,
            definition.commitments(),
            &penalization_store,
            definition.composition(),
            physical_store,
        )?;
        // Compute penalty expression
        let penalty =
            build_penalization_expression(&penalization_store, *definition.penalty());
        // Compute revenue expression
        let mut revenue = 0.0.into_expression();
        for market in markets.iter() {
            revenue += market.revenue();
        }
        Ok(Self {
            id: *definition.id(),
            markets,
            bid_store,
            constraints,
            penalization_store,
            revenue,
            penalty,
        })
    }
    /// Makes new bids one-sided per slot and links the perimeter net position
    /// (net commitment, new bids and imbalance) to its assets.
    fn build_exclusivity_and_repartition_constraints(
        vars: &mut ProblemVariables,
        constraints: &mut Vec<Constraint>,
        bid_store: &TimeSeries<BidVariables>,
        commitments: &TimeSeries<PowerCommitment>,
        penalization_store: &TimeSeries<BidVariables>,
        composition: &[Uuid],
        physical_store: &PhysicalStore,
    ) -> crate::Result<()> {
        let (sum_rated_input_power, sum_rated_output_power) =
            physical_store.maximum_physical_limits(composition)?;
        // Largest useful trade on either side in one slot: cancelling the
        // committed position and swinging to the opposite physical limit.
        let big_m = sum_rated_input_power + sum_rated_output_power;
        // Commitments are netted per slot over the whole grid, so they
        // provide the time index.
        for commitment in commitments.data() {
            let dt = &commitment.start_at;
            let bid_variables = bid_store.at(dt)?;
            // Exclusivity binds new bids only, so the perimeter can't buy and
            // sell the same slot for phantom earnings, while it can still
            // unwind its committed position.
            constraints.extend(bid_variables.exclusivity_constraint(vars, big_m, big_m));
            let penalization_variables = penalization_store.at(dt)?;
            // Perimeter net position: net commitment, new bids and imbalance.
            let mut perimeter_net =
                (commitment.input_power.0 - commitment.output_power.0).into_expression();
            perimeter_net.add_mul(1.0, bid_variables.input_power());
            perimeter_net.add_mul(-1.0, bid_variables.output_power());
            perimeter_net.add_mul(1.0, penalization_variables.input_power());
            perimeter_net.add_mul(-1.0, penalization_variables.output_power());
            let mut sum_asset_net = 0.0.into_expression();
            for asset_id in composition.iter() {
                let asset = physical_store.get(asset_id)?;
                sum_asset_net += asset.input_power_at(dt)? - asset.output_power_at(dt)?;
            }
            constraints.push(constraint!(sum_asset_net == perimeter_net));
        }
        Ok(())
    }
    /// Moves the perimeter and its markets constraints out, leaving them empty.
    pub(crate) fn take_constraints(&mut self) -> impl Iterator<Item = Constraint> {
        std::mem::take(&mut self.constraints)
            .into_iter()
            .chain(self.markets.iter_mut().flat_map(Market::take_constraints))
    }
}
// endregion: Balance Responsible Party Perimeter

// region:  Wholesale Perimeter
/// Container of all balancing responsible party perimeters.
/// Assets belonging to at most one of them is guaranteed by the problem.
#[derive(Debug)]
pub struct WholesalePerimeter {
    brp_perimeters: Vec<BrpPerimeter>,
    revenue: Expression,
    penalty: Expression,
}

impl WholesalePerimeter {
    pub(crate) fn try_new(
        definitions: &[BrpDefinition],
        physical_store: &PhysicalStore,
        env: &BuildEnv<'_>,
        vars: &mut ProblemVariables,
    ) -> crate::Result<Self> {
        let brp_perimeters = definitions
            .iter()
            .map(|definition| {
                BrpPerimeter::try_new(definition, physical_store, env, vars)
            })
            .collect::<crate::Result<Vec<BrpPerimeter>>>()?;
        // Compute overall revenue and penalty.
        let mut revenue = 0.0.into_expression();
        let mut penalty = 0.0.into_expression();
        for brp in brp_perimeters.iter() {
            revenue += &brp.revenue;
            penalty += &brp.penalty;
        }
        Ok(Self { brp_perimeters, revenue, penalty })
    }
    pub fn brp_perimeters(&self) -> &[BrpPerimeter] {
        &self.brp_perimeters
    }
    pub(crate) fn revenue(&self) -> &Expression {
        &self.revenue
    }
    pub(crate) fn penalty(&self) -> &Expression {
        &self.penalty
    }
    /// Moves every perimeter's constraints out, leaving them empty.
    pub(crate) fn take_constraints(&mut self) -> impl Iterator<Item = Constraint> {
        self.brp_perimeters.iter_mut().flat_map(BrpPerimeter::take_constraints)
    }
}

// endregion:  Wholesale Perimeter

// region: Solution conversion
impl BrpPerimeter {
    pub(crate) fn to_solution(
        &self,
        solution: &impl good_lp::Solution,
    ) -> crate::Result<BrpSolution> {
        let revenue_brp = solution.eval(&self.revenue);
        let mut market_solutions =
            Vec::<MarketSolution<WholesaleMarketType, WholesaleBid>>::with_capacity(
                self.markets.len(),
            );
        for market in self.markets.iter() {
            let revenue_market = solution.eval(market.revenue());
            let market_solution = MarketSolution {
                market_type: WholesaleMarketType::try_from(*market.market_type())?,
                solution: SolutionWithRevenue {
                    revenue: revenue_market,
                    series: market
                        .bid_variables()
                        .map(|bid_variables| bid_variables.to_wholesale_bid(solution))
                        .collect(),
                    step: *market.step(),
                },
            };
            market_solutions.push(market_solution)
        }
        let shortages = self
            .penalization_store
            .data()
            .iter()
            .map(|bid_variables| bid_variables.to_wholesale_bid(solution))
            .collect();
        Ok(BrpSolution {
            id: self.id,
            revenue: revenue_brp,
            markets: market_solutions,
            penalty: solution.eval(&self.penalty),
            shortages,
        })
    }
}

// endregion: Solution conversion
