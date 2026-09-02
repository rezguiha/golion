use super::market_type::MarketType;
use crate::countries::Countries;
use jiff::Span;
use jiff::civil::Time;

#[derive(Debug)]
pub enum MarketError {
    // -- Temporality
    InvalidDeltaDayValue { value: Span },
    InvalidTimeDefinedBound { start_time: Time, end_time: Time, delta_start_end: Span },
    EmptyContinuousBiddingBound,
    // -- config
    NotImplemented { market: MarketType, country: Countries },
}
