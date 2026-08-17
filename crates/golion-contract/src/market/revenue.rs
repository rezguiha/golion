/// Market Revenue Series
use garde::Validate;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
/// Wholesale revenue series.
/// For example:
///     -spot intraday continuous
///     - spot day ahead
///     - intraday auction 1,2,3
/// In markets where we have same price for sell and buy.
/// These might be equal. Even in that case, someone might
/// add a sell and buy margin to simulate the price you are going
/// to go under or upper to execute the trade.
#[derive(Debug, Deserialize, Serialize, Validate, TypedBuilder)]
pub struct SimplifiedWholesaleRevenue {
    /// Reference datetime for future estimated revenue.
    #[garde(skip)]
    pub start_at: Timestamp,
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
#[derive(Debug, Deserialize, Serialize, Validate, TypedBuilder)]
pub struct SimplifiedAncillaryRevenue {
    /// Reference datetime for future estimated revenue.
    #[garde(skip)]
    pub start_at: Timestamp,
    /// Estimated market upward revenue in €/kW.
    #[garde(range(min = 0.0))]
    pub sell_revenue: f64,
    /// Estimated market downward revenue in €/kW.
    #[garde(range(min = 0.0))]
    pub buy_revenue: f64,
}

/// A mapping between markets and their corresponding data
/// models. An enum on market type has been chosen as ground
/// work to potentially having different revenue structs depending
/// on each market. This could be simplified later on if project
/// does not evolve in that direction.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "market")]
pub enum AncillaryRevenueSeries {
    Afrr(Vec<SimplifiedAncillaryRevenue>),
    Fcr(Vec<SimplifiedAncillaryRevenue>),
    AfrrFree(Vec<SimplifiedAncillaryRevenue>),
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "market")]
pub enum WholeSaleRevenueSeries {
    SpotDayAhead(Vec<SimplifiedAncillaryRevenue>),
    IntradayAuction(Vec<SimplifiedAncillaryRevenue>),
    IntradayContinuous(Vec<SimplifiedAncillaryRevenue>),
}
