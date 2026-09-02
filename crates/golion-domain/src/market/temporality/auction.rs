use crate::market::temporality::interval::{DeltaDays, TimeDefinedInterval};
use jiff::Span;
use jiff::civil::Time;
use jiff::tz::TimeZone;

/// Market temporality where an auction is held at fixed times
/// and where bidding starts and ends at fixed times.

#[derive(Debug)]
pub struct StaticAuctionTemporality {
    pub open_interval: TimeDefinedInterval,
    pub bidding_interval: TimeDefinedInterval,
    pub delta_close_bidding_start: DeltaDays,
    pub timezone: TimeZone,
}

impl StaticAuctionTemporality {
    pub fn try_new(
        open_interval: TimeDefinedInterval,
        bidding_interval: TimeDefinedInterval,
        delta_close_bidding_start: Span,
        timezone: &str,
    ) -> crate::Result<Self> {
        let timezone = TimeZone::get(timezone)?;
        let delta_close_bidding_start_days: DeltaDays =
            delta_close_bidding_start.try_into()?;
        Ok(Self {
            open_interval,
            bidding_interval,
            delta_close_bidding_start: delta_close_bidding_start_days,
            timezone,
        })
    }
}

/// Market temporality where an auction is dynamic and defined by a neutralization
/// period from reference and bidding times with reference as anchor.

#[derive(Debug)]
pub struct DynamicAuctionTemporality {
    /// Represents available bidding period.
    pub bidding_interval: TimeDefinedInterval,
    /// Represents the minimal time separating
    /// reference time and the next available bid.
    /// This may represent liquidity constraints
    /// on certain markets in certain countries or
    /// a regulatory constraint like in ancillary services
    /// about gate opening and closures.
    pub neutralization_delay: Span,
    pub timezone: TimeZone,
}
impl DynamicAuctionTemporality {
    pub fn try_new(
        bidding_interval: TimeDefinedInterval,
        neutralization_delay: Span,
        timezone: &str,
    ) -> crate::Result<Self> {
        let timezone = TimeZone::get(timezone)?;
        Ok(Self { bidding_interval, neutralization_delay, timezone })
    }
}

/// Market temporality where an auction represents a continuous bidding like
/// intraday continuous where bidding is always open either on current day or or next
/// day. The neutralization delay represents liquidity constraints.

#[derive(Debug)]
pub struct ContinuousAuctionTemporality {
    pub next_day_gate_open_time: Time,
    pub neutralization_delay: Span,
    pub timezone: TimeZone,
}

impl ContinuousAuctionTemporality {
    pub fn try_new(
        next_day_gate_open_hour: i8,
        next_day_gate_open_minute: i8,
        neutralization_delay: Span,
        timezone: &str,
    ) -> crate::Result<Self> {
        let timezone = TimeZone::get(timezone)?;
        let next_day_gate_open_time =
            Time::new(next_day_gate_open_hour, next_day_gate_open_minute, 0, 0)?;
        Ok(Self { next_day_gate_open_time, neutralization_delay, timezone })
    }
}
