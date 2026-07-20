/// Market Revenue Series
use chrono::{DateTime, Utc};
use garde::Validate;
use serde::Deserialize;
use typed_builder::TypedBuilder;

/// Symmetric revenue series. This concerns wholesale markets
/// where buy and sell prices are the same.
/// For example:
///     - spot day ahead
///     - intraday auction 1,2,3
#[derive(Debug, Deserialize, Validate, TypedBuilder)]
pub struct SymmetricSeries {
    /// Reference datetime for future estimated revenue.
    #[garde(skip)]
    pub start_at: DateTime<Utc>,
    /// Forecasted market price in €/kWh
    #[garde(skip)]
    pub price: f64,
}

/// Asymmetric revenue series. This concerns wholesale markets
/// where buy and sell prices/revenue are  not the same.
/// For example:
///     -spot intraday continuous
#[derive(Debug, Deserialize, Validate, TypedBuilder)]
pub struct AsymmetricSeries {
    /// Reference datetime for future estimated revenue.
    #[garde(skip)]
    pub start_at: DateTime<Utc>,
    /// Forecasted market sell price  in €/kWh
    #[garde(skip)]
    pub sell_price: f64,
    /// Forecasted market buy price in €/kWh
    #[garde(skip)]
    pub buy_price: f64,
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
    pub start_at: DateTime<Utc>,
    /// Estimated market upward revenue in €/kW.
    #[garde(range(min = 0.0))]
    pub sell_revenue: f64,
    /// Estimated market downward revenue in €/kW.
    #[garde(range(min = 0.0))]
    pub buy_revenue: f64,
}

// Several revenue series will be defined here later on
// one for aFRR energy free bidding which is an energy part ancillary
// bidding.
// One for capacity auctions like aFRR capacicty and one for FCR markets
