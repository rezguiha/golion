use super::support::{
    aggregate_bidding_and_commitments, build_penalization_expression,
    build_penalization_variables,
};
use crate::{
    market::{core::Market, variables::BidVariables},
    model::BuildEnv,
    physical::PhysicalStore,
};
use golion_domain::{
    problem::definition::ReserveDefinition, temporal::series::TimeSeries,
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
    /// Ancillary Service
    market: Market,
    /// Perimeter level aggregated bidding and commitments variables.
    variable_store: TimeSeries<BidVariables>,
    /// Repartition variables/expressions per asset of ancillary commitments
    /// and bidding over each timestamp in time index
    repartition: HashMap<Uuid, TimeSeries<BidVariables>>,
    /// Container for constraints for repartition.
    constraints: Vec<Constraint>,
    /// Reserve level penalization in order to avoid violations.
    /// This represents the imbalance.
    penalization_store: TimeSeries<BidVariables>,
    /// Reserve level revenue expression including penalization.
    revenue: Expression,
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
        let variable_store = aggregate_bidding_and_commitments(
            horizon.timestamps(),
            horizon.grid(),
            definition.commitments(),
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
        let mut constraints: Vec<Constraint> = Vec::new();
        Self::create_repartition_constraints(
            horizon.timestamps(),
            &mut constraints,
            &variable_store,
            &repartition,
        )?;
        // Build penalization Variables.
        let penalization_store = build_penalization_variables(horizon, vars)?;
        // Build revenue expression
        let revenue =
            build_penalization_expression(&penalization_store, *definition.penalty())
                + market.revenue();
        Ok(Self {
            market,
            variable_store,
            repartition,
            constraints,
            penalization_store,
            revenue,
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
    /// Set perimeter aggregation being equal to sum of asset
    /// repartition of that market
    fn create_repartition_constraints(
        time_index: &[Timestamp],
        constraints: &mut Vec<Constraint>,
        perimeter_aggregation: &TimeSeries<BidVariables>,
        asset_level_repartition_variables: &HashMap<Uuid, TimeSeries<BidVariables>>,
    ) -> crate::Result<()> {
        for dt in time_index.iter() {
            let perimeter = perimeter_aggregation.at(dt)?;
            let mut perimeter_input_power = 0.0.into_expression();
            perimeter_input_power += perimeter.input_power();
            let mut perimeter_output_power = 0.0.into_expression();
            perimeter_output_power += perimeter.output_power();
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
        // Compute Overall Revnue.
        let mut revenue = 0.0.into_expression();
        for reserve in reserve_perimeters.iter() {
            revenue += &reserve.revenue;
        }
        Ok(Self { reserve_perimeters, constraints, revenue })
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
    pub fn reserve_perimeters(&self) -> &[ReservePerimeter] {
        &self.reserve_perimeters
    }
    pub(crate) fn revenue(&self) -> &Expression {
        &self.revenue
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
