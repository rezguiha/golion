use super::market_type::MarketType;
use crate::countries::Countries;
use crate::market::bid::ProductSpecifications;
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
}
