use super::market_type::MarketType;
use crate::countries::Countries;
use crate::market::bid::ProductSpecifications;
use jiff::civil::Time;
use jiff::{SignedDuration, Span, Timestamp};
use std::collections::HashSet;
#[derive(Debug)]
pub enum MarketError {
    // -- Temporality
    InvalidDeltaDayValue {
        value: Span,
    },
    InvalidTimeDefinedBound {
        start_time: Time,
        end_time: Time,
        delta_start_end: Span,
    },
    BidStepNotMultipleGridStep {
        bid_step: SignedDuration,
        grid_step: SignedDuration,
    },
    BidWindowNotValid {
        start: Timestamp,
        end: Timestamp,
    },
    // -- Specification
    InvalidProduct {
        market: MarketType,
        country: Countries,
        product: ProductSpecifications,
        possible_products: HashSet<ProductSpecifications>,
    },

    // -- Revenue
    MissingMarketRevenue {
        market: MarketType,
        country: Countries,
    },

    // -- config
    NotImplemented {
        market: MarketType,
        country: Countries,
    },

    // -- Market type conversion
    NotWholesaleMarket {
        market: MarketType,
    },
    NotAncillaryMarket {
        market: MarketType,
    },

    // -- Activation
    InvalidActivationRatioError {
        upward: f64,
        downward: f64,
    },
}
