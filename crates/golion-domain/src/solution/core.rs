use crate::{
    market::{
        commitment::{EnergyCommitment, PowerCommitment},
        market_type::{AncillaryMarketType, WholesaleMarketType},
    },
    temporal::{series::TimeStampedUtc, step::MinuteStep},
};

#[derive(Debug)]
pub struct SolutionWithRevenue<D: TimeStampedUtc> {
    pub revenue: f64,
    pub step: MinuteStep,
    pub series: Vec<D>,
}
#[derive(Debug)]
pub struct MarketSolution<T, D: TimeStampedUtc> {
    pub market_type: T,
    pub solution: SolutionWithRevenue<D>,
}

#[derive(Debug)]
pub struct BrpSolution {
    pub revenue: f64,
    pub markets: Vec<MarketSolution<WholesaleMarketType, EnergyCommitment>>,
    pub penalty: SolutionWithRevenue<EnergyCommitment>,
}

#[derive(Debug)]
pub struct ReserveSolution {
    pub solution: MarketSolution<AncillaryMarketType, PowerCommitment>,
    pub penalty: SolutionWithRevenue<PowerCommitment>,
}

#[derive(Debug)]
pub struct OptimizationSolution {
    pub ancillary: Vec<ReserveSolution>,
    pub wholesale: Vec<BrpSolution>,
}
