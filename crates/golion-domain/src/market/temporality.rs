use jiff::civil::Time;
use jiff::tz::TimeZone;
use jiff::{RoundMode, Span, Timestamp, ToSpan, Unit, Zoned, ZonedRound};

use crate::market::bid::BidSpecs;
// region: Delta in Days

/// This struct is used to represent the time separating auction closure and bidding
/// start. For example : for day ahead market the bidding is by definition on the next day
/// from auction closure.
/// It is also used to represent the time separating auction open and close times. For example
/// for intraday third auction.

#[derive(Debug)]
pub struct DeltaDays(Span);

impl DeltaDays {
    // Getter method.
    fn value(&self) -> &Span {
        &self.0
    }
}

#[derive(Debug)]
pub enum MarketTemporalityError {
    InvalidDeltaDayValue { value: Span },
    InvalidIntervalBound { start_time: Time, end_time: Time, delta_start_end: Span },
}

impl TryFrom<Span> for DeltaDays {
    type Error = crate::Error;
    fn try_from(value: Span) -> crate::Result<Self> {
        [0.days().fieldwise(), 1.days().fieldwise()]
            .contains(&value.fieldwise())
            .then_some(DeltaDays(value))
            .ok_or(MarketTemporalityError::InvalidDeltaDayValue { value }.into())
    }
}

// endregion: Delta in Days

// region: Static Auction

#[derive(Debug)]
pub struct TimeDefinedInterval {
    pub start_time: Time,
    pub end_time: Time,
    pub delta_start_end: DeltaDays,
}
impl TimeDefinedInterval {
    fn try_new(
        start_time: Time,
        end_time: Time,
        delta_start_end: Span,
    ) -> crate::Result<Self> {
        let delta_start_end_days: DeltaDays = delta_start_end.try_into()?;
        if (start_time > end_time) & (delta_start_end_days.0.get_days() == 0) {
            Err(MarketTemporalityError::InvalidIntervalBound {
                start_time,
                end_time,
                delta_start_end: delta_start_end_days.0,
            }
            .into())
        } else {
            Ok(TimeDefinedInterval {
                start_time,
                end_time,
                delta_start_end: delta_start_end_days,
            })
        }
    }
}

/// Market temporality where an auction is held at fixed times
/// and where bidding starts and ends at fixed times.

#[derive(Debug)]
pub struct StaticAuctionTemporality {
    open_interval: TimeDefinedInterval,
    bidding_interval: TimeDefinedInterval,
    delta_close_bidding_start: DeltaDays,
    timezone: TimeZone,
}

impl StaticAuctionTemporality {
    fn try_new(
        open_interval: TimeDefinedInterval,
        bidding_interval: TimeDefinedInterval,
        delta_close_bidding_start: Span,
        timezone: TimeZone,
    ) -> crate::Result<Self> {
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

// endregion: Static Auction

// region: Dynamic Auction

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
// endregion: Dynamic Auction

// region: Bidding Time Boundaries and its trait implementations

/// Represents the time boundaries of bidding that are available
/// at a particular reference time (optimization run time or reference time
/// in case of backtesting).
#[derive(Debug)]
pub struct BidTimeBounds {
    start_at: Timestamp,
    end_at: Timestamp,
}
/// Defines the ability to compute bidding start and end relative
/// to a reference timestamp which will be in our case the run time
/// of the process.
pub trait ToBidTimeBounds {
    fn to_bid_time_bounds(
        &self,
        reference_time: &Timestamp,
        bid_specifications: &BidSpecs,
    ) -> crate::Result<BidTimeBounds>;
}

/// Convenience method that adds a delta in days and sets the time to the new
/// zoned datetime.
fn add_and_set_time(reference: &Zoned, delta: &Span, time: Time) -> crate::Result<Zoned> {
    let new = reference.checked_add(delta).and_then(|dt| dt.with().time(time).build())?;
    Ok(new)
}
impl ToBidTimeBounds for StaticAuctionTemporality {
    fn to_bid_time_bounds(
        &self,
        reference_time: &Timestamp,
        bid_specifications: &BidSpecs,
    ) -> crate::Result<BidTimeBounds> {
        // We are using clone here on timezone as it is cheap to clone
        // and to_zoned requires to pass ownership of timezone.
        let zoned_reference_time = reference_time.to_zoned(self.timezone.clone());
        let zoned_open_at =
            zoned_reference_time.with().time(self.open_interval.start_time).build()?;
        let zoned_close_at = add_and_set_time(
            &zoned_open_at,
            self.open_interval.delta_start_end.value(),
            self.open_interval.end_time,
        )?;
        let zoned_bidding_start = add_and_set_time(
            &zoned_close_at,
            self.delta_close_bidding_start.value(),
            self.bidding_interval.start_time,
        )?;
        let zoned_bidding_end = add_and_set_time(
            &zoned_bidding_start,
            self.bidding_interval.delta_start_end.value(),
            self.bidding_interval.end_time,
        )?;
        // Round start_at to next multiple of bid granularity
        let start_at = zoned_bidding_start.round(
            ZonedRound::new()
                .smallest(Unit::Minute)
                .increment(bid_specifications.step.duration().as_mins())
                .mode(RoundMode::Expand),
        )?;
        // Round end_at to previous multiple of bid granularity
        // as convention is [start,end[ to be able to fit the last
        // bid in period.
        let end_at = zoned_bidding_end.round(
            ZonedRound::new()
                .smallest(Unit::Minute)
                .increment(bid_specifications.step.duration().as_mins())
                .mode(RoundMode::Trunc),
        )?;
        Ok(BidTimeBounds { start_at: start_at.into(), end_at: end_at.into() })
    }
}

impl ToBidTimeBounds for DynamicAuctionTemporality {
    fn to_bid_time_bounds(
        &self,
        reference_time: &Timestamp,
        bid_specifications: &BidSpecs,
    ) -> crate::Result<BidTimeBounds> {
        let zoned_reference_time = reference_time.to_zoned(self.timezone.clone());
        let zoned_bidding_start = add_and_set_time(
            &zoned_reference_time,
            &self.neutralization_delay,
            self.bidding_interval.start_time,
        )?;
        let zoned_bidding_end = add_and_set_time(
            &zoned_bidding_start,
            self.bidding_interval.delta_start_end.value(),
            self.bidding_interval.end_time,
        )?;
        // Round to next multiple of bid granularity.
        let start_at = zoned_bidding_start.round(
            ZonedRound::new()
                .smallest(Unit::Minute)
                .increment(bid_specifications.step.duration().as_mins())
                .mode(RoundMode::Ceil),
        )?;

        let end_at = zoned_bidding_end.round(
            ZonedRound::new()
                .smallest(Unit::Minute)
                .increment(bid_specifications.step.duration().as_mins())
                .mode(RoundMode::Trunc),
        )?;
        Ok(BidTimeBounds { start_at: start_at.into(), end_at: end_at.into() })
    }
}
// endregion: Bidding Time Boundaries and its trait implementations
