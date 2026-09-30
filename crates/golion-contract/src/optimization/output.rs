/// Optimization results sent back after solve.
use crate::market::commitments::AncillaryCommitment;
use crate::market::series::MarketSeries;
use garde::Validate;
use golion_domain::market::bid::{AncillaryBid, WholesaleBid};
use golion_domain::market::market_type::{AncillaryMarketType, WholesaleMarketType};
use golion_domain::solution::{
    BrpSolution, MarketSolution, OptimizationSolution, ReserveSolution,
};
use golion_domain::temporal::series::TimeStampedUtc;
use jiff::Timestamp;
use serde::Serialize;
use uuid::Uuid;

// region: Optimization Output

#[derive(Debug, Serialize)]
pub struct MarketOutput<M, V: Validate<Context = ()>> {
    /// Revenue earned on the market, in €.
    pub revenue: f64,
    #[serde(flatten)]
    pub series: MarketSeries<M, V>,
}

/// Wholesale markets' bid expressed in kWh.
#[derive(Debug, Serialize, Validate)]
pub struct WholesaleBidContract {
    #[garde(skip)]
    pub start_at: Timestamp,
    /// Sell energy in kWh.
    #[garde(range(min = 0.0))]
    pub sell_energy: f64,
    /// Buy energy in kWh.
    #[garde(range(min = 0.0))]
    pub buy_energy: f64,
}
/// Deviation of a balance responsible party perimeter from its traded
/// position over a slot, in kWh.
pub type Imbalance = WholesaleBidContract;
/// Results of a balance responsible party perimeter.
#[derive(Debug, Serialize)]
pub struct BrpPerimeterOutput {
    pub id: Uuid,
    /// Total revenue of the perimeter, penalty included, in €.
    pub revenue: f64,
    pub markets: Vec<MarketOutput<WholesaleMarketType, WholesaleBidContract>>,
    /// Penalty paid for the shortages, in €.
    pub penalty: f64,
    /// Imbalance of the perimeter per slot in kWh.
    pub shortages: Vec<Imbalance>,
}
///  Ancillary markets' bid in kW.
pub type AncillaryBidContract = AncillaryCommitment;
pub type ReserveShortfall = AncillaryCommitment;
/// Results of a reserve perimeter on its ancillary service.
#[derive(Debug, Serialize)]
pub struct ReservePerimeterOutput {
    pub id: Uuid,
    #[serde(flatten)]
    pub market: MarketOutput<AncillaryMarketType, AncillaryBidContract>,
    /// Penalty paid for the shortages, in €.
    pub penalty: f64,
    /// Shortfall of the perimeter on its commitments per slot in kW.
    pub shortages: Vec<ReserveShortfall>,
}

/// Optimization results per perimeter.
#[derive(Debug, Serialize)]
pub struct OptimizationOutput {
    pub wholesale: Vec<BrpPerimeterOutput>,
    pub ancillary: Vec<ReservePerimeterOutput>,
}
// endregion: Optimization Output

// region: Domain Conversion
impl<M, D, V> From<MarketSolution<M, D>> for MarketOutput<M, V>
where
    D: TimeStampedUtc,
    V: Validate<Context = ()> + From<D>,
{
    fn from(value: MarketSolution<M, D>) -> Self {
        let MarketSolution { market_type, solution } = value;
        Self {
            revenue: solution.revenue,
            series: MarketSeries {
                market: market_type,
                values: solution.series.into_iter().map(V::from).collect(),
            },
        }
    }
}

impl From<WholesaleBid> for WholesaleBidContract {
    fn from(value: WholesaleBid) -> Self {
        Self {
            start_at: value.start_at,
            sell_energy: value.sell_energy.0,
            buy_energy: value.buy_energy.0,
        }
    }
}

impl From<AncillaryBid> for AncillaryBidContract {
    fn from(value: AncillaryBid) -> Self {
        Self {
            start_at: value.start_at,
            upward_power: value.upward_power.0,
            downward_power: value.downward_power.0,
        }
    }
}

impl From<BrpSolution> for BrpPerimeterOutput {
    fn from(value: BrpSolution) -> Self {
        let BrpSolution { id, revenue, markets, penalty, shortages } = value;
        Self {
            id,
            revenue,
            markets: markets.into_iter().map(MarketOutput::from).collect(),
            penalty,
            shortages: shortages.into_iter().map(Imbalance::from).collect(),
        }
    }
}

impl From<ReserveSolution> for ReservePerimeterOutput {
    fn from(value: ReserveSolution) -> Self {
        let ReserveSolution { id, solution, penalty, shortages } = value;
        Self {
            id,
            market: solution.into(),
            penalty,
            shortages: shortages.into_iter().map(ReserveShortfall::from).collect(),
        }
    }
}

impl From<OptimizationSolution> for OptimizationOutput {
    fn from(value: OptimizationSolution) -> Self {
        let OptimizationSolution { ancillary, wholesale } = value;
        Self {
            wholesale: wholesale.into_iter().map(BrpPerimeterOutput::from).collect(),
            ancillary: ancillary.into_iter().map(ReservePerimeterOutput::from).collect(),
        }
    }
}
// endregion: Domain Conversion
