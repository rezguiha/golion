use jiff::civil::Time;
use jiff::tz::TimeZone;
use jiff::{Span, Timestamp, ToSpan};
// region: Delta in Days

/// This struct is used to represent the time separating auction closure and bidding
/// start. For example : for day ahead market the bidding is by definition on the next day
/// from auction closure.
/// It is also used to represent the time separating auction open and close times. For example
/// for intraday third auction.

#[derive(Debug)]
pub struct DeltaDays(Span);

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

// region: Static Auction temporality representation and its structs.
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

// endregion: Static Auction temporality representation and its structs.

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
    ) -> crate::Result<BidTimeBounds>;
}

impl ToBidTimeBounds for StaticAuctionTemporality {
    fn to_bid_time_bounds(
        &self,
        reference_time: &Timestamp,
    ) -> crate::Result<BidTimeBounds> {
        // We are using clone here on timezone as it is cheap to clone
        // and to_zoned requires to pass ownership of timezone.
        let zoned_reference_time = reference_time.to_zoned(self.timezone.clone());
        let zoned_open_at =
            zoned_reference_time.with().time(self.open_interval.start_time).build()?;
        // The use of saturating_add is fine here as we can at most add 1 or 2 days.
        let zoned_close_at = zoned_open_at
            .saturating_add(self.open_interval.delta_start_end.0)
            .with()
            .time(self.open_interval.end_time)
            .build()?;
        let zoned_bidding_start = zoned_close_at
            .saturating_add(self.delta_close_bidding_start.0)
            .with()
            .time(self.bidding_interval.start_time)
            .build()?;
        let zoned_bidding_end = zoned_bidding_start
            .saturating_add(self.bidding_interval.delta_start_end.0)
            .with()
            .time(self.bidding_interval.end_time)
            .build()?;
        Ok(BidTimeBounds {
            start_at: zoned_bidding_start.into(),
            end_at: zoned_bidding_end.into(),
        })
    }
}

// endregion: Bidding Time Boundaries and its trait implementations
