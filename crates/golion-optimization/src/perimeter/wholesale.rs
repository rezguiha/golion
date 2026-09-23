use super::support::{aggregate_bidding_and_commitments, build_penalization_variables};
use crate::{
    market::{core::Market, variables::BidVariables},
    model::BuildEnv,
    physical::PhysicalStore,
};
use golion_domain::{problem::definition::BrpDefinition, temporal::series::TimeSeries};
use good_lp::{Constraint, IntoAffineExpression, ProblemVariables, constraint};
use jiff::Timestamp;
use uuid::Uuid;

// region: Balance Responsible Party Perimeter
#[derive(Debug)]
pub struct BrpPerimeter {
    /// List of wholesale markets to bid on
    markets: Vec<Market>,
    /// Perimeter level aggregated bidding and commitments variables.
    variable_store: TimeSeries<BidVariables>,
    /// Perimeter constraints.
    constraints: Vec<Constraint>,
    /// Perimeter level penalization in order to avoid violations.
    /// This represents the imbalance.
    penalization_store: TimeSeries<BidVariables>,
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
        let markets: Vec<Market> = definition
            .markets()
            .iter()
            .map(|market_specs| Market::try_new(market_specs, env, vars))
            .collect::<crate::Result<_>>()?;
        let variable_store = aggregate_bidding_and_commitments(
            horizon.timestamps(),
            horizon.grid(),
            definition.commitments(),
            &markets,
        )?;

        // Add Repartition Constraints
        let mut constraints =
            Vec::<Constraint>::with_capacity(horizon.timestamps().len());
        Self::build_repartition_constraints(
            &mut constraints,
            &variable_store,
            definition.composition(),
            horizon.timestamps(),
            physical_store,
        )?;
        let penalization_store = build_penalization_variables(horizon, vars)?;
        Ok(Self { markets, variable_store, constraints, penalization_store })
    }
    fn build_repartition_constraints(
        constraints: &mut Vec<Constraint>,
        variable_store: &TimeSeries<BidVariables>,
        composition: &[Uuid],
        time_index: &[Timestamp],
        physical_store: &PhysicalStore,
    ) -> crate::Result<()> {
        for dt in time_index.iter() {
            let perimeter_variables = variable_store.at(dt)?;
            // We defined perimeter net as a 0 expression and add to it input power
            // and subtract output power to avoid moving values behind them.
            // This enables us to avoid that.
            let mut perimeter_net = 0.0.into_expression();
            perimeter_net.add_mul(1.0, perimeter_variables.input_power());
            perimeter_net.add_mul(-1.0, perimeter_variables.output_power());
            let mut sum_asset_net = 0.0.into_expression();
            for asset_id in composition.iter() {
                let asset = physical_store.get(asset_id)?;
                sum_asset_net += asset.input_power_at(dt)? - asset.output_power_at(dt)?;
            }
            constraints.push(constraint!(sum_asset_net == perimeter_net));
        }
        Ok(())
    }
}
// endregion: Balance Responsible Party Perimeter

// region:  Wholesale Perimeter
/// Container of all balancing responsible party perimeters.
/// Assets belonging to at most one of them is guaranteed by the problem.
#[derive(Debug)]
pub struct WholesalePerimeter {
    brp_perimeters: Vec<BrpPerimeter>,
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
            .collect::<crate::Result<_>>()?;
        Ok(Self { brp_perimeters })
    }
    pub fn brp_perimeters(&self) -> &[BrpPerimeter] {
        &self.brp_perimeters
    }
}
// endregion:  Wholesale Perimeter
