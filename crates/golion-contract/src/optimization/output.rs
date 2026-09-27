/// Optimization results sent back after solve. Market results reuse the input
/// commitment structs, so they can be fed back as the commitments of a
/// following run. Penalties have their own rows, as they are deviations and
/// never commitments.
use crate::market::commitments::{AncillaryCommitment, WholesaleCommitment};
use crate::market::series::MarketSeries;
use garde::Validate;
use golion_domain::market::commitment::{EnergyCommitment, PowerCommitment};
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

/// Deviation of a balance responsible party perimeter from its traded
/// position over a slot, in kWh.
#[derive(Debug, Serialize)]
pub struct Imbalance {
    pub start_at: Timestamp,
    /// Energy missing from the perimeter position, as its assets consumed
    /// more than it traded in kWh.
    pub short: f64,
    /// Energy in excess of the perimeter position, as its assets produced
    /// more than it traded in kWh.
    pub long: f64,
}

/// Reserve a perimeter committed to but could not back with its assets over a
/// slot, in kW.
#[derive(Debug, Serialize)]
pub struct ReserveShortfall {
    pub start_at: Timestamp,
    /// Missing discharge reserve in kW.
    pub upward_power: f64,
    /// Missing charge reserve in kW.
    pub downward_power: f64,
}

/// Results of a balance responsible party perimeter.
#[derive(Debug, Serialize)]
pub struct BrpPerimeterOutput {
    pub id: Uuid,
    /// Total revenue of the perimeter, penalty included, in €.
    pub revenue: f64,
    pub markets: Vec<MarketOutput<WholesaleMarketType, WholesaleCommitment>>,
    /// Penalty paid for the shortages, in €.
    pub penalty: f64,
    /// Imbalance of the perimeter per slot in kW.
    pub shortages: Vec<Imbalance>,
}

/// Results of a reserve perimeter on its ancillary service.
#[derive(Debug, Serialize)]
pub struct ReservePerimeterOutput {
    pub id: Uuid,
    #[serde(flatten)]
    pub market: MarketOutput<AncillaryMarketType, AncillaryCommitment>,
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

impl From<EnergyCommitment> for Imbalance {
    fn from(value: EnergyCommitment) -> Self {
        Self { start_at: value.start_at, short: value.input.0, long: value.output.0 }
    }
}

impl From<PowerCommitment> for ReserveShortfall {
    fn from(value: PowerCommitment) -> Self {
        Self {
            start_at: value.start_at,
            upward_power: value.output.0,
            downward_power: value.input.0,
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
