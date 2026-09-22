use std::{collections::HashMap, ops::Not};

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

// region: Balance Responsible Party Perimeter
#[derive(Debug)]
pub struct BrpPerimeter {
    /// List of wholesale markets to bid on
    markets: Vec<Market>,
    /// List of ids of assets inside the perimeter
    composition: Vec<Uuid>,
    /// List of commitments on all wholesale markets
    /// for the perimeter.
    commitments: Vec<Commitment>,
    /// Perimeter level aggregated bidding and commitments variables.
    variable_store: TimeSeries<BidVariables>,
    /// Perimeter constraints.
    constraints: Vec<Constraint>,
}

impl BrpPerimeter {
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
// endregion: Balance Responsible Party Perimeter

// region:  Wholesale Perimeter
/// Container of all balancing responsible party perimeters.
#[derive(Debug)]
pub enum WholesalePerimeterError {
    AssetInMultipleBrps { asset_ids: Vec<Uuid> },
}
#[derive(Debug)]
pub struct WholesalePerimeter {
    brp_perimeters: Vec<BrpPerimeter>,
}

impl WholesalePerimeter {
    pub fn try_new(brp_perimeters: Vec<BrpPerimeter>) -> crate::Result<Self> {
        // Make sure an asset can be in at most one balance responsible party
        // perimeter.
        let mut counter = HashMap::new();
        for brp_perimeter in brp_perimeters.iter() {
            for id in brp_perimeter.composition.iter() {
                *counter.entry(*id).or_insert(0) += 1;
            }
        }
        let duplicated: Vec<_> = counter
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(id, _)| id)
            .collect();
        if duplicated.is_empty().not() {
            Err(WholesalePerimeterError::AssetInMultipleBrps { asset_ids: duplicated }
                .into())
        } else {
            Ok(Self { brp_perimeters })
        }
    }
    pub fn brp_perimeters(&self) -> &[BrpPerimeter] {
        &self.brp_perimeters
    }
}
// endregion:  Wholesale Perimeter
