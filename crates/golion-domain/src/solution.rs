use crate::{
    asset::state::{BessState, OtherAssetState},
    market::{
        bid::{AncillaryBid, WholesaleBid},
        market_type::{AncillaryMarketType, WholesaleMarketType},
    },
    temporal::{series::TimeStampedUtc, step::Step},
};
use serde::Serialize;
use uuid::Uuid;
#[derive(Debug)]
pub enum PhysicalStates {
    Bess { asset_id: Uuid, series: Vec<BessState> },
    Ccgt { asset_id: Uuid, series: Vec<OtherAssetState> },
    Ren { asset_id: Uuid, series: Vec<OtherAssetState> },
}

#[derive(Debug)]
pub struct MarketSolution<T, D: TimeStampedUtc> {
    pub market_type: T,
    pub revenue: f64,
    pub step: Step,
    pub series: Vec<D>,
}

#[derive(Debug)]
pub struct BrpSolution {
    pub id: Uuid,
    pub revenue: f64,
    pub penalty: f64,
    pub markets: Vec<MarketSolution<WholesaleMarketType, WholesaleBid>>,
    pub shortages: Vec<WholesaleBid>,
}

#[derive(Debug)]
pub struct ReserveSolution {
    pub id: Uuid,
    pub penalty: f64,
    pub solution: MarketSolution<AncillaryMarketType, AncillaryBid>,
    pub shortages: Vec<AncillaryBid>,
}

#[derive(Debug)]
pub struct OptimizationSolution {
    pub ancillary: Vec<ReserveSolution>,
    pub wholesale: Vec<BrpSolution>,
    pub physical: Vec<PhysicalStates>,
}

#[derive(Debug, Serialize)]
pub enum Optimality {
    Optimal,
    WithinGap,
}

#[derive(Debug, Serialize)]
pub enum FailureReason {
    LimitReached,
    Infeasible,
    Unbounded,
}

#[derive(Debug)]
pub enum OptimizationOutcome {
    Success { optimality: Optimality, solution: OptimizationSolution },
    Failure { reason: FailureReason },
}
