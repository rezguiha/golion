use super::market_type::MarketType;
use crate::countries::Countries;
use crate::market::bid::BidSpecs;
use jiff::Span;
use jiff::civil::Time;
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
    EmptyContinuousBiddingBound,
    // -- Specification
    InvalidProduct {
        market: MarketType,
        country: Countries,
        product: BidSpecs,
        possible_products: HashSet<BidSpecs>,
    },

    // -- config
    NotImplemented {
        market: MarketType,
        country: Countries,
    },
}
