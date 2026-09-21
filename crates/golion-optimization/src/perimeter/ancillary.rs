use super::support::aggregate_bidding_and_commitments;
use crate::market::{core::Market, variables::BidVariables};
use golion_domain::{
    market::commitment::Commitment,
    temporal::{grid::RegularTimeGrid, series::TimeSeries},
};
use good_lp::{Constraint, IntoAffineExpression, ProblemVariables, variable};
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
    /// Repartition variables/expressions per asset of ancillary commitments and bidding
    /// over each timestamp in time index
    repartition: HashMap<Uuid, TimeSeries<BidVariables>>,
    /// Container for constraints for repartition.
    constraints: Vec<Constraint>,
    /// Commitments of the perimeter for the ancillary service.
    commitments: Vec<Commitment>,
}

impl ReservePerimeter {
    pub fn try_new(
        market: Market,
        composition: Vec<Uuid>,
        commitments: Vec<Commitment>,
        time_index: &[Timestamp],
        time_grid: &RegularTimeGrid,
        vars: &mut ProblemVariables,
    ) -> crate::Result<Self> {
        let variable_store = aggregate_bidding_and_commitments(
            time_index,
            time_grid,
            &commitments,
            std::slice::from_ref(&market),
        )?;
        let repartition = composition
            .into_iter()
            .map(|id| {
                Self::create_asset_repartition_variables(
                    vars,
                    time_index,
                    time_grid.step().duration(),
                )
                .map(|series| (id, series))
            })
            .collect::<crate::Result<HashMap<Uuid, TimeSeries<BidVariables>>>>()?;
        Ok(Self { market, variable_store, repartition, constraints: vec![], commitments })
    }
    /// Creates
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
}
/// Container of all ancillary service perimeters.
#[derive(Debug)]
pub struct AncillaryPerimeter {
    pub(crate) reserve_perimiters: Vec<ReservePerimeter>,
}
