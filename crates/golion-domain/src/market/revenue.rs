/// Market Revenue Series
use chrono::{DateTime, Utc};
use garde::Validate;
use serde::Deserialize;
use typed_builder::TypedBuilder;

use crate::series::HasTimeStamps;

/// Symmetric revenue series. This concerns wholesale markets
/// where buy and sell prices are the same.
/// For example:
///     - spot day ahead
///     - intraday auction 1,2,3
#[derive(Debug, Deserialize, Validate, TypedBuilder)]
pub struct SymmetricSeries {
    /// Reference datetime for future estimated revenue.
    #[garde(skip)]
    start_at: Vec<DateTime<Utc>>,
    /// Forecasted market price in €/kWh
    #[garde(skip)]
    price: Vec<f64>,
}
impl HasTimeStamps for SymmetricSeries {
    fn timestamps(&self) -> &[DateTime<Utc>] {
        &self.start_at
    }
}

/// Asymmetric revenue series. This concerns wholesale markets
/// where buy and sell prices/revenue are  not the same.
/// For example:
///     -spot intraday continuous
#[derive(Debug, Deserialize, Validate, TypedBuilder)]
pub struct AsymmetricSeries {
    /// Reference datetime for future estimated revenue.
    #[garde(skip)]
    start_at: Vec<DateTime<Utc>>,
    /// Forecasted market sell price  in €/kWh
    #[garde(skip)]
    sell_price: Vec<f64>,
    /// Forecasted market buy price in €/kWh
    #[garde(skip)]
    buy_price: Vec<f64>,
}
impl HasTimeStamps for AsymmetricSeries {
    fn timestamps(&self) -> &[DateTime<Utc>] {
        &self.start_at
    }
}

/// A simplified version of revenue model for ancillary service
/// markets that represents estimated revnues for upward and
/// downward bids per kW bid. For example:
///     - aFRR free bidding
///     - aFRR capacity bidding
///     - FCR
#[derive(Debug, Deserialize, Validate, TypedBuilder)]
pub struct SimplifiedAncillarySeries {
    /// Reference datetime for future estimated revenue.
    #[garde(skip)]
    start_at: Vec<DateTime<Utc>>,
    /// Estimated market upward revenue in €/kW.
    #[garde(inner(range(min = 0.0)))]
    sell_revenue: Vec<f64>,
    /// Estimated market downward revenue in €/kW.
    #[garde(inner(range(min = 0.0)))]
    buy_revenue: Vec<f64>,
}
impl HasTimeStamps for SimplifiedAncillarySeries {
    fn timestamps(&self) -> &[DateTime<Utc>] {
        &self.start_at
    }
}

// Several revenue series will be defined here later on
// one for aFRR energy free bidding which is an energy part ancillary
// bidding.
// One for capacity auctions like aFRR capacicty and one for FCR markets
