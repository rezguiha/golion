use jiff::civil::Time;
use jiff::tz::TimeZone;
use jiff::{RoundMode, Span, Timestamp, ToSpan, Unit, Zoned, ZonedRound};

use crate::market::bid::BidSpecs;
use crate::temporal::step::MinuteStep;
// region: Errors

#[derive(Debug)]
pub enum MarketTemporalityError {
    InvalidDeltaDayValue { value: Span },
    InvalidTimeDefinedBound { start_time: Time, end_time: Time, delta_start_end: Span },
    EmptyContinuousBiddingBound,
}
// endregion: Errors

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

impl TryFrom<Span> for DeltaDays {
    type Error = crate::Error;
    fn try_from(value: Span) -> crate::Result<Self> {
        [0.days().fieldwise(), 1.days().fieldwise()]
            .contains(&value.fieldwise())
            .then_some(DeltaDays(value))
            .ok_or(MarketTemporalityError::InvalidDeltaDayValue { value }.into())
    }
}

/// Represents an interval defined by a starting time and ending time
/// and a delta expressed in days.
/// The times will be associated with a reference date and the delta will be applied
/// between start and end to determine starting and ending datetimes.
///
/// The convention used here is [start,end].
///
/// For example:
///     To represent day ahead bidding interval:
///         start_time: 00:00
///         end_time: 23:59
///
/// Make sure to use for end_time a value that represents correctly the end.
/// For example if bidding ends at 11:00:
/// Make sure to put 10:59 to avoid including 11:00 in your representation
/// as it might lead to different results.
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
            Err(MarketTemporalityError::InvalidTimeDefinedBound {
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

// endregion: Delta in Days

// region: Static Auction

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

// region: Continuous Auction

#[derive(Debug)]
pub struct ContinuousAuctionTemporality {
    pub next_day_gate_open_time: Time,
    pub neutralization_delay: Span,
    pub timezone: TimeZone,
}

// endregion: Continuous Auction

// region: Bidding Time Boundaries and its trait implementations

/// Represents the time boundaries of bidding that are available
/// at a particular reference time (optimization run time or reference time
/// in case of backtesting).
///
/// Available bids are those where their timestamp dt in starting convention
/// is in [start_at,end_at[.
#[derive(Debug)]
pub struct BidTimeBounds {
    start_at: Timestamp,
    end_at: Timestamp,
}
/// Defines the ability to compute bidding start and end relative
/// to a reference timestamp which will be in our case the run time
/// of the process.
pub trait ToBidTimeBounds {
    /// Returns `None` when the computed bidding window does not contain a
    /// single full bid step (e.g. it is narrower than the step, or empty).
    fn to_bid_time_bounds(
        &self,
        reference_time: &Timestamp,
        bid_specifications: &BidSpecs,
    ) -> crate::Result<Option<BidTimeBounds>>;
}

/// Convenience method that adds a delta in days and sets the time to the new
/// zoned datetime.
fn add_and_set_time(reference: &Zoned, delta: &Span, time: Time) -> crate::Result<Zoned> {
    let new = reference.checked_add(delta).and_then(|dt| dt.with().time(time).build())?;
    Ok(new)
}
/// Adapts bidding start and end to fit bid step which is
/// expressed in minutes.
/// We round up start to next multiple of bid step and end
/// to the previous one as convention is as starting convention
/// to make sure to have full steps inside interval.
///
/// It returns None when the window between start and end is too narrow
/// and smaller than the bid step. In this case, there is no bidding interval
/// to be returned.
fn fit_bounds_to_bid_step(
    start: &Zoned,
    end: &Zoned,
    bid_step: &MinuteStep,
) -> crate::Result<Option<(Zoned, Zoned)>> {
    let minutes = bid_step.duration().as_mins();
    let new_start = start.round(
        ZonedRound::new()
            .smallest(Unit::Minute)
            .increment(minutes)
            .mode(RoundMode::Expand),
    )?;
    let new_end = end.round(
        ZonedRound::new()
            .smallest(Unit::Minute)
            .increment(minutes)
            .mode(RoundMode::Trunc),
    )?;
    if new_start >= new_end {
        return Ok(None);
    }
    Ok(Some((new_start, new_end)))
}
impl ToBidTimeBounds for StaticAuctionTemporality {
    fn to_bid_time_bounds(
        &self,
        reference_time: &Timestamp,
        bid_specifications: &BidSpecs,
    ) -> crate::Result<Option<BidTimeBounds>> {
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
        let bounds = fit_bounds_to_bid_step(
            &zoned_bidding_start,
            &zoned_bidding_end,
            &bid_specifications.step,
        )?;

        Ok(bounds.map(|(start_at, end_at)| BidTimeBounds {
            start_at: start_at.into(),
            end_at: end_at.into(),
        }))
    }
}

impl ToBidTimeBounds for DynamicAuctionTemporality {
    fn to_bid_time_bounds(
        &self,
        reference_time: &Timestamp,
        bid_specifications: &BidSpecs,
    ) -> crate::Result<Option<BidTimeBounds>> {
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
        let bounds = fit_bounds_to_bid_step(
            &zoned_bidding_start,
            &zoned_bidding_end,
            &bid_specifications.step,
        )?;
        Ok(bounds.map(|(start_at, end_at)| BidTimeBounds {
            start_at: start_at.into(),
            end_at: end_at.into(),
        }))
    }
}

impl ToBidTimeBounds for ContinuousAuctionTemporality {
    fn to_bid_time_bounds(
        &self,
        reference_time: &Timestamp,
        bid_specifications: &BidSpecs,
    ) -> crate::Result<Option<BidTimeBounds>> {
        let zoned_reference_time = reference_time.to_zoned(self.timezone.clone());
        // Compute same day bidding bounds.
        let same_day_end = zoned_reference_time.end_of_day()?;

        let same_day_start =
            zoned_reference_time.checked_add(self.neutralization_delay)?;
        let same_day_bounds = &fit_bounds_to_bid_step(
            &same_day_start,
            &same_day_end,
            &bid_specifications.step,
        )?;
        // Compute next day bidding bounds.
        let next_bidding_day_start =
            zoned_reference_time.tomorrow().and_then(|dt| dt.start_of_day())?;
        let next_day_bidding_start = match next_bidding_day_start {
            dt if dt >= same_day_start => dt,
            // At reference times at the end of the day, adding the neutralization
            // delay will exceed the scope of the current day into next one and
            // we need to exclude the relevant times from bidding of next.
            _ => same_day_start,
        };
        let next_day_bidding_end = next_day_bidding_start.end_of_day()?;

        let next_day_bounds = fit_bounds_to_bid_step(
            &next_day_bidding_start,
            &next_day_bidding_end,
            &bid_specifications.step,
        )?;

        match (same_day_bounds, next_day_bounds) {
            // Continuous bidding will always have either bidding on same day
            // or next day. If there is none on both days there was an issue in computation.
            (None, None) => {
                Err(MarketTemporalityError::EmptyContinuousBiddingBound {}.into())
            }
            // If next bidding not still available return same day bidding bounds.
            (Some((same_start_at, same_end_at)), None) => Ok(Some(BidTimeBounds {
                start_at: same_start_at.into(),
                end_at: same_end_at.into(),
            })),
            // If same and next bidding are available return combination of the two
            // as bidding is continuous in time.
            (Some((same_start_at, _)), Some((_, next_end_at))) => {
                Ok(Some(BidTimeBounds {
                    start_at: same_start_at.into(),
                    end_at: next_end_at.into(),
                }))
            }
            // If only next day bidding is available return it. This may happen at the boundary
            // of the two days.
            (None, Some((next_start_at, next_end_at))) => Ok(Some(BidTimeBounds {
                start_at: next_start_at.into(),
                end_at: next_end_at.into(),
            })),
        }
    }
}
// endregion: Bidding Time Boundaries and its trait implementations

// region: Tests
#[cfg(test)]
mod tests {
    use jiff::{SignedDuration, Timestamp, ToSpan, civil::Time, tz::TimeZone};

    use crate::market::bid::{BidSpecs, KiloWattIncrement};
    use crate::market::temporality::{
        ContinuousAuctionTemporality, DynamicAuctionTemporality, TimeDefinedInterval,
        ToBidTimeBounds,
    };
    use crate::temporal::step::MinuteStep;

    fn bid_specs(step_minutes: i64) -> BidSpecs {
        BidSpecs {
            step: MinuteStep::try_from(SignedDuration::from_mins(step_minutes)).unwrap(),
            increment: KiloWattIncrement::from(100),
        }
    }

    #[test]
    fn dynamic_window_less_than_bid_step() {
        let bidding_interval = TimeDefinedInterval {
            start_time: Time::new(10, 0, 0, 0).unwrap(),
            end_time: Time::new(10, 3, 0, 0).unwrap(),
            delta_start_end: 0.days().try_into().unwrap(),
        };
        let timezone = TimeZone::get("CET").unwrap();
        let neutralization_delay = 0.minutes();
        let dynamic_auction = DynamicAuctionTemporality {
            bidding_interval,
            neutralization_delay,
            timezone,
        };

        // 2024-01-15T09:00:00Z == 10:00 CET.
        let reference_time: Timestamp = "2024-01-15T09:00:00Z".parse().unwrap();
        let bounds =
            dynamic_auction.to_bid_time_bounds(&reference_time, &bid_specs(15)).unwrap();

        assert!(bounds.is_none());
    }
    #[test]
    fn dynamic_single_available_bid() {
        // 9:50-10:20 CET is one full 15-minute step (10:00-10:15).
        let bidding_interval = TimeDefinedInterval {
            start_time: Time::new(9, 50, 0, 0).unwrap(),
            end_time: Time::new(10, 20, 0, 0).unwrap(),
            delta_start_end: 0.days().try_into().unwrap(),
        };
        let timezone = TimeZone::get("CET").unwrap();
        let neutralization_delay = 0.minutes();
        let dynamic_auction = DynamicAuctionTemporality {
            bidding_interval,
            neutralization_delay,
            timezone,
        };

        // 2024-01-15T09:00:00Z == 10:00 CET.
        let reference_time: Timestamp = "2024-01-15T09:00:00Z".parse().unwrap();
        let bounds = dynamic_auction
            .to_bid_time_bounds(&reference_time, &bid_specs(15))
            .unwrap()
            .unwrap();

        // 10:00 CET is already on the step boundary == 09:00Z.
        assert_eq!(bounds.start_at, "2024-01-15T09:00:00Z".parse().unwrap());
        // 10:20 CET truncates down to 10:15 CET == 09:15Z.
        assert_eq!(bounds.end_at, "2024-01-15T09:15:00Z".parse().unwrap());
    }

    fn continuous_test(
        reference_time: Timestamp,
        expected_bidding_start: Timestamp,
        expected_bidding_end: Timestamp,
    ) {
        let timezone = TimeZone::get("CET").unwrap();
        let next_day_gate_open_time = Time::new(15, 0, 0, 0).unwrap();
        let neutralization_delay = 2.hours();
        let continuous_auction = ContinuousAuctionTemporality {
            neutralization_delay,
            next_day_gate_open_time,
            timezone,
        };
        let bounds = continuous_auction
            .to_bid_time_bounds(&reference_time, &bid_specs(15))
            .unwrap()
            .unwrap();
        assert_eq!(bounds.start_at, expected_bidding_start);
        assert_eq!(bounds.end_at, expected_bidding_end);
    }
    #[test]
    fn continuous_same_day_only() {
        let reference_time: Timestamp = "2024-01-15T09:00:00Z".parse().unwrap();
        let expected_bidding_start = "2024-01-15T11:00:00Z".parse().unwrap();
        let expected_bidding_end = "2024-01-16T22:45:00Z".parse().unwrap();
        continuous_test(reference_time, expected_bidding_start, expected_bidding_end);
    }
    #[test]
    fn continuous_next_only() {
        let reference_time: Timestamp = "2024-01-15T22:55:00Z".parse().unwrap();
        let expected_bidding_start = "2024-01-16T01:00:00Z".parse().unwrap();
        let expected_bidding_end = "2024-01-16T22:45:00Z".parse().unwrap();
        continuous_test(reference_time, expected_bidding_start, expected_bidding_end);
    }
    #[test]
    fn continuous_both_days() {
        let reference_time: Timestamp = "2024-01-15T16:00:00Z".parse().unwrap();
        let expected_bidding_start = "2024-01-15T18:00:00Z".parse().unwrap();
        let expected_bidding_end = "2024-01-16T22:45:00Z".parse().unwrap();
        continuous_test(reference_time, expected_bidding_start, expected_bidding_end);
    }
}
// endregion: Tests
