use super::support::aggregate_bidding_and_commitments;
use crate::{
    market::{core::Market, variables::BidVariables},
    physical::PhysicalStore,
};
use golion_domain::{
    market::commitment::Commitment,
    temporal::{grid::RegularTimeGrid, series::TimeSeries},
};
use good_lp::{Constraint, IntoAffineExpression, constraint};
use jiff::Timestamp;
use uuid::Uuid;
#[derive(Debug)]
pub struct WholesalePerimeter {
    /// List of markets to bid on
    pub(crate) markets: Vec<Market>,
    /// List of ids of assets inside the perimeter
    pub(crate) composition: Vec<Uuid>,
    /// List of commitments on all wholesale markets
    /// for the perimeter.
    pub(crate) commitments: Vec<Commitment>,
    /// Perimeter level aggregated bidding and commitments variables.
    pub(crate) variable_store: TimeSeries<BidVariables>,
    /// Perimeter constraints.
    pub(crate) constraints: Vec<Constraint>,
}

impl WholesalePerimeter {
    pub fn try_new(
        time_index: &[Timestamp],
        time_grid: &RegularTimeGrid,
        commitments: Vec<Commitment>,
        markets: Vec<Market>,
        composition: Vec<Uuid>,
        physical_store: &PhysicalStore,
    ) -> crate::Result<Self> {
        let variable_store = aggregate_bidding_and_commitments(
            time_index,
            time_grid,
            &commitments,
            &markets,
        )?;

        // Add Repartition Constraints
        let mut constraints = Vec::<Constraint>::with_capacity(time_index.len());
        Self::build_repartition_constraints(
            &mut constraints,
            &variable_store,
            &composition,
            time_index,
            physical_store,
        )?;
        Ok(Self { markets, composition, commitments, variable_store, constraints })
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
