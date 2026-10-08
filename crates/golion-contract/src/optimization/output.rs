/// Optimization results sent back after solve.
use crate::market::commitments::AncillaryCommitment;
use crate::market::series::MarketSeries;
use garde::Validate;
use golion_domain::asset::state::{BessState, OtherAssetState};
use golion_domain::market::bid::{AncillaryBid, WholesaleBid};
use golion_domain::market::market_type::{AncillaryMarketType, WholesaleMarketType};
use golion_domain::solution::{
    BrpSolution, MarketSolution, OptimizationSolution, PhysicalStates, ReserveSolution,
};
use golion_domain::temporal::series::TimeStampedUtc;
use jiff::Timestamp;
use serde::Serialize;
use uuid::Uuid;

// region: Market Output

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
// endregion: Market Output

// region: Asset Output
/// State of a battery over a slot.
#[derive(Debug, Serialize)]
pub struct BessStateOutput {
    pub start_at: Timestamp,
    /// Net power in kW, positive when charging and negative when discharging.
    pub dispatch: f64,
    /// State of charge reached at the end of the slot, in kWh.
    pub soc: f64,
}

/// State of a non-storage asset over a slot.
#[derive(Debug, Serialize)]
pub struct OtherAssetStateOutput {
    pub start_at: Timestamp,
    /// Output power in kW.
    pub dispatch: f64,
}

/// Physical results of an asset, tagged by `asset_type` like its input.
#[derive(Debug, Serialize)]
#[serde(tag = "asset_type")]
pub enum AssetOutput {
    #[serde(rename = "BESS")]
    Bess { asset_id: Uuid, series: Vec<BessStateOutput> },
    #[serde(rename = "CCGT")]
    GasTurbine { asset_id: Uuid, series: Vec<OtherAssetStateOutput> },
    #[serde(rename = "RENEWABLE")]
    Renewable { asset_id: Uuid, series: Vec<OtherAssetStateOutput> },
}
// endregion: Asset Output

// region: Optimization Output

/// Optimization results per perimeter and per asset.
#[derive(Debug, Serialize)]
pub struct OptimizationOutput {
    pub wholesale: Vec<BrpPerimeterOutput>,
    pub ancillary: Vec<ReservePerimeterOutput>,
    pub physical: Vec<AssetOutput>,
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
            shortages: shortages
                .into_iter()
                .filter(|x| x.buy_energy.0 != 0.0 || x.sell_energy.0 != 0.0)
                .map(Imbalance::from)
                .collect(),
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
            shortages: shortages
                .into_iter()
                .filter(|x| x.downward_power.0 != 0.0 || x.upward_power.0 != 0.0)
                .map(ReserveShortfall::from)
                .collect(),
        }
    }
}

impl From<BessState> for BessStateOutput {
    fn from(value: BessState) -> Self {
        Self { start_at: value.start_at, dispatch: value.dispatch.0, soc: value.soc.0 }
    }
}

impl From<OtherAssetState> for OtherAssetStateOutput {
    fn from(value: OtherAssetState) -> Self {
        Self { start_at: value.start_at, dispatch: value.dispatch.0 }
    }
}

impl From<PhysicalStates> for AssetOutput {
    fn from(value: PhysicalStates) -> Self {
        match value {
            PhysicalStates::Bess { asset_id, series } => Self::Bess {
                asset_id,
                series: series.into_iter().map(BessStateOutput::from).collect(),
            },
            PhysicalStates::Ccgt { asset_id, series } => Self::GasTurbine {
                asset_id,
                series: series.into_iter().map(OtherAssetStateOutput::from).collect(),
            },
            PhysicalStates::Ren { asset_id, series } => Self::Renewable {
                asset_id,
                series: series.into_iter().map(OtherAssetStateOutput::from).collect(),
            },
        }
    }
}

impl From<OptimizationSolution> for OptimizationOutput {
    fn from(value: OptimizationSolution) -> Self {
        let OptimizationSolution { ancillary, wholesale, physical } = value;
        Self {
            wholesale: wholesale.into_iter().map(BrpPerimeterOutput::from).collect(),
            ancillary: ancillary.into_iter().map(ReservePerimeterOutput::from).collect(),
            physical: physical.into_iter().map(AssetOutput::from).collect(),
        }
    }
}
// endregion: Domain Conversion
