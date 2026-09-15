/// Market Revenue Series. These series enable us to compute
/// the revenue generated from the market.
use garde::Validate;
use golion_domain::{
    market::{
        market_type::{AncillaryMarketType, WholesaleMarketType},
        revenue::{Revenue, RevenueUnit},
    },
    temporal::series::TimeStampedUtc,
};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::market::series::MarketSeries;

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(untagged)]
pub enum AncillaryRevenue {
    /// A simplified version of revenue model for ancillary service
    /// markets that represents estimated revnues for upward and
    /// downward bids per kW bid. For example:
    ///     - aFRR free bidding
    ///     - aFRR capacity bidding
    ///     - FCR
    SimplifiedAncillaryRevenue {
        /// Reference datetime for future estimated revenue.
        #[garde(skip)]
        start_at: Timestamp,
        /// Estimated market upward revenue in €/MW.
        #[garde(range(min = 0.0))]
        sell_revenue: f64,
        /// Estimated market downward revenue in €/MW.
        #[garde(range(min = 0.0))]
        buy_revenue: f64,
    },
}

impl TimeStampedUtc for AncillaryRevenue {
    fn start_at(&self) -> &Timestamp {
        match self {
            Self::SimplifiedAncillaryRevenue { start_at, .. } => start_at,
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(untagged)]
pub enum WholesaleRevenue {
    /// Wholesale revenue series.
    /// For example:
    ///     -spot intraday continuous
    ///     - spot day ahead
    ///     - intraday auction 1,2,3
    /// In markets where we have same price for sell and buy.
    /// These might be equal. Even in that case, someone might
    /// add a sell and buy margin to simulate the price you are going
    /// to go under or upper to execute the trade.
    SimplifiedWholesaleRevenue {
        /// Reference datetime for future estimated revenue.
        #[garde(skip)]
        start_at: Timestamp,
        /// Forecasted market sell price  in €/MWh
        #[garde(skip)]
        sell_price: f64,
        /// Forecasted market buy price in €/MWh
        #[garde(skip)]
        buy_price: f64,
    },
}
impl TimeStampedUtc for WholesaleRevenue {
    fn start_at(&self) -> &Timestamp {
        match self {
            Self::SimplifiedWholesaleRevenue { start_at, .. } => start_at,
        }
    }
}
/// A mapping between markets and their corresponding data
/// models. An enum on market type has been chosen as ground
/// work to potentially having different revenue structs depending
/// on each market. This could be simplified later on if project
/// does not evolve in that direction.
pub(crate) type AncillaryRevenueSeries =
    Vec<MarketSeries<AncillaryMarketType, AncillaryRevenue>>;
pub(crate) type WholesaleRevenueSeries =
    Vec<MarketSeries<WholesaleMarketType, WholesaleRevenue>>;
// region: Domain conversion
/// Converts revenue input values to revenue values
impl From<&AncillaryRevenue> for Revenue {
    fn from(value: &AncillaryRevenue) -> Self {
        match value {
            AncillaryRevenue::SimplifiedAncillaryRevenue {
                start_at,
                sell_revenue,
                buy_revenue,
            } => Self {
                // No sign or transformation needed here as we
                // receive revenue estimation and not forecasted market
                // prices.
                unit: RevenueUnit::PerKiloWatt,
                input_revenue: buy_revenue / 1000.0,
                output_revenue: sell_revenue / 1000.0,
                start_at: *start_at,
            },
        }
    }
}

impl From<&WholesaleRevenue> for Revenue {
    fn from(value: &WholesaleRevenue) -> Self {
        match value {
            WholesaleRevenue::SimplifiedWholesaleRevenue {
                start_at,
                sell_price,
                buy_price,
            } => Self {
                // No sign or transformation needed here as we
                // receive revenue estimation and not forecasted market
                // prices.
                unit: RevenueUnit::PerKiloWattHour,
                input_revenue: buy_price / 1000.0,
                output_revenue: sell_price / 1000.0,
                start_at: *start_at,
            },
        }
    }
}
// endregion: Domain conversion
