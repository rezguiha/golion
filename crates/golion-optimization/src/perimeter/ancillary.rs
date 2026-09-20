use crate::market::{core::Market, variables::BidVariables};
use golion_domain::{market::commitment::Commitment, temporal::series::TimeSeries};
use good_lp::Constraint;
use std::collections::HashMap;
use uuid::Uuid;

/// Perimeter for the ancillary service containing
/// assets certified together for it.
#[derive(Debug)]
pub struct ReservePerimeter {
    /// Ancillary Service
    pub(crate) market: Market,
    /// Repartition variables/expressions per asset of ancillary commitments and bidding
    /// over each timestamp in time index
    pub(crate) repartition: HashMap<Uuid, TimeSeries<BidVariables>>,
    /// Container for constraints for repartition.
    pub(crate) constraints: Vec<Constraint>,
    /// Commitments of the perimeter for the ancillary service.
    pub(crate) commitments: Vec<Commitment>,
}
/// Container of all ancillary service perimeters.
#[derive(Debug)]
pub struct AncillaryPerimeter {
    pub(crate) reserve_perimiters: Vec<ReservePerimeter>,
}
