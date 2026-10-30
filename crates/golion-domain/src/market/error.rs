use super::market_type::MarketType;
use crate::countries::Countries;
use crate::market::market_type::WholesaleMarketType;
use crate::market::product::ProductDef;
use crate::temporal::step::Step;
use jiff::civil::Time;
use jiff::{SignedDuration, Span, Timestamp};
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
    InvalidWholesaleProduct {
        market: WholesaleMarketType,
        country: Countries,
        product_step: Step,
        possible_product_steps: Vec<Step>,
    },

    // -- Revenue
    MissingMarketRevenue {
        product: ProductDef,
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
