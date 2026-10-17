use jiff::civil::Time;
use jiff::{RoundMode, Span, Timestamp, Unit, Zoned, ZonedRound};

use crate::market::bid::ProductSpecifications;
use crate::market::error::MarketError;
use crate::market::temporality::auction::{
    ContinuousAuctionTemporality, DynamicAuctionTemporality, StaticAuctionTemporality,
};
use crate::temporal::grid::RegularTimeGrid;
use crate::temporal::step::MinuteStep;
// region: Bidding Time Boundaries and its trait implementations

/// Defines the ability to compute bidding start and end relative
/// to a reference timestamp which will be in our case the run time
/// of the process.
pub trait ToBidTimeBounds {
    /// Returns `None` when the computed bidding window does not contain a
    /// single full bid step (e.g. it is narrower than the step, or empty).
    fn to_bid_time_bounds(
        &self,
        reference_time: &Timestamp,
        product_specifications: &ProductSpecifications,
    ) -> crate::Result<Option<RegularTimeGrid>>;
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
        product_specifications: &ProductSpecifications,
    ) -> crate::Result<Option<RegularTimeGrid>> {
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
            &product_specifications.step,
        )?;

        bounds
            .map(|(start_at, end_at)| {
                RegularTimeGrid::try_new_start_end(
                    start_at.into(),
                    product_specifications.step,
                    end_at.into(),
                )
            })
            .transpose()
    }
}

impl ToBidTimeBounds for DynamicAuctionTemporality {
    fn to_bid_time_bounds(
        &self,
        reference_time: &Timestamp,
        product_specifications: &ProductSpecifications,
    ) -> crate::Result<Option<RegularTimeGrid>> {
        let zoned_reference_time = reference_time.to_zoned(self.timezone.clone());
        // Compute earliest from reference after neutralization delay?
        let zoned_bidding_earliest =
            zoned_reference_time.checked_add(self.neutralization_delay)?;
        // Compute bidding window start taking as reference the day on which
        // the earliest available bid is on. This enables us to handle edge cases
        // around midnight.
        let zoned_window_start = zoned_bidding_earliest
            .with()
            .time(self.bidding_interval.start_time)
            .build()?;
        // Compute end always on the basis of window start.
        let zoned_bidding_end = add_and_set_time(
            &zoned_window_start,
            self.bidding_interval.delta_start_end.value(),
            self.bidding_interval.end_time,
        )?;

        // Take the latest between earliest and window start.
        let zoned_bidding_start = zoned_bidding_earliest.max(zoned_window_start);

        let bounds = fit_bounds_to_bid_step(
            &zoned_bidding_start,
            &zoned_bidding_end,
            &product_specifications.step,
        )?;
        bounds
            .map(|(start_at, end_at)| {
                RegularTimeGrid::try_new_start_end(
                    start_at.into(),
                    product_specifications.step,
                    end_at.into(),
                )
            })
            .transpose()
    }
}

impl ToBidTimeBounds for ContinuousAuctionTemporality {
    fn to_bid_time_bounds(
        &self,
        reference_time: &Timestamp,
        product_specifications: &ProductSpecifications,
    ) -> crate::Result<Option<RegularTimeGrid>> {
        let zoned_reference_time = reference_time.to_zoned(self.timezone.clone());
        // Compute same day bidding bounds.
        let same_day_end = zoned_reference_time.end_of_day()?;

        let same_day_start =
            zoned_reference_time.checked_add(self.neutralization_delay)?;
        let same_day_bounds = &fit_bounds_to_bid_step(
            &same_day_start,
            &same_day_end,
            &product_specifications.step,
        )?;
        // Compute next day bidding bounds. Next day products only open for
        // trading at the next day gate opening time of the reference day.
        let next_day_gate_open_at =
            zoned_reference_time.with().time(self.next_day_gate_open_time).build()?;
        let next_day_bounds = if zoned_reference_time >= next_day_gate_open_at {
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
            fit_bounds_to_bid_step(
                &next_day_bidding_start,
                &next_day_bidding_end,
                &product_specifications.step,
            )?
        } else {
            None
        };

        match (same_day_bounds, next_day_bounds) {
            // Continuous bidding will always have either bidding on same day
            // or next day. If there is none on both days there was an issue in computation.
            (None, None) => Err(MarketError::EmptyContinuousBiddingBound {}.into()),
            // If next bidding not still available return same day bidding bounds.
            (Some((same_start_at, same_end_at)), None) => {
                RegularTimeGrid::try_new_start_end(
                    same_start_at.into(),
                    product_specifications.step,
                    same_end_at.into(),
                )
                .map(Some)
            }
            // If same and next bidding are available return combination of the two
            // as bidding is continuous in time.
            (Some((same_start_at, _)), Some((_, next_end_at))) => {
                RegularTimeGrid::try_new_start_end(
                    same_start_at.into(),
                    product_specifications.step,
                    next_end_at.into(),
                )
                .map(Some)
            }
            // If only next day bidding is available return it. This may happen at the boundary
            // of the two days.
            (None, Some((next_start_at, next_end_at))) => {
                RegularTimeGrid::try_new_start_end(
                    next_start_at.into(),
                    product_specifications.step,
                    next_end_at.into(),
                )
                .map(Some)
            }
        }
    }
}
// endregion: Bidding Time Boundaries and its trait implementations

// region: Tests
#[cfg(test)]
mod tests {
    use jiff::{SignedDuration, Timestamp, ToSpan};

    use crate::market::bid::{KiloWattIncrement, ProductSpecifications};
    use crate::market::temporality::{
        auction::{ContinuousAuctionTemporality, DynamicAuctionTemporality},
        bid_time_bounds::ToBidTimeBounds,
        interval::TimeDefinedInterval,
    };
    use crate::temporal::step::MinuteStep;

    fn product_specifications(step_minutes: i64) -> ProductSpecifications {
        ProductSpecifications {
            step: MinuteStep::try_from(SignedDuration::from_mins(step_minutes)).unwrap(),
            increment: KiloWattIncrement::from(100),
        }
    }

    /// Checks the bidding bounds of an auction at a reference time for
    /// 15-minute products, `None` meaning no full bid step is available.
    fn bounds_test(
        auction: &impl ToBidTimeBounds,
        reference_time: Timestamp,
        expected_bounds: Option<(Timestamp, Timestamp)>,
    ) {
        let bounds = auction
            .to_bid_time_bounds(&reference_time, &product_specifications(15))
            .unwrap();
        assert_eq!(bounds.map(|grid| (*grid.start(), *grid.end())), expected_bounds);
    }
    #[test]
    fn dynamic_window_less_than_bid_step() {
        let auction = DynamicAuctionTemporality::try_new(
            TimeDefinedInterval::try_new(10, 0, 10, 3, 0.days()).unwrap(),
            0.minutes(),
            "CET",
        )
        .unwrap();
        // 10:00 CET, the 10:00-10:03 CET window holds no full 15-minute step.
        bounds_test(&auction, "2024-01-15T09:00:00Z".parse().unwrap(), None);
    }
    #[test]
    fn dynamic_single_available_bid() {
        let auction = DynamicAuctionTemporality::try_new(
            TimeDefinedInterval::try_new(9, 50, 10, 20, 0.days()).unwrap(),
            0.minutes(),
            "CET",
        )
        .unwrap();
        // 10:00 CET, 9:50-10:20 CET is one full 15-minute step: 10:00-10:15 CET.
        bounds_test(
            &auction,
            "2024-01-15T09:00:00Z".parse().unwrap(),
            Some((
                "2024-01-15T09:00:00Z".parse().unwrap(),
                "2024-01-15T09:15:00Z".parse().unwrap(),
            )),
        );
    }
    #[test]
    fn dynamic_applies_neutralization_delay() {
        let auction = DynamicAuctionTemporality::try_new(
            TimeDefinedInterval::try_new(0, 0, 23, 59, 0.days()).unwrap(),
            30.minutes(),
            "CET",
        )
        .unwrap();
        // 10:00 CET + 30 minutes == 10:30 CET, 23:59 CET truncates to 23:45 CET.
        bounds_test(
            &auction,
            "2024-01-15T09:00:00Z".parse().unwrap(),
            Some((
                "2024-01-15T09:30:00Z".parse().unwrap(),
                "2024-01-15T22:45:00Z".parse().unwrap(),
            )),
        );
    }
    #[test]
    fn dynamic_delay_crosses_midnight() {
        let auction = DynamicAuctionTemporality::try_new(
            TimeDefinedInterval::try_new(0, 0, 23, 59, 0.days()).unwrap(),
            30.minutes(),
            "CET",
        )
        .unwrap();
        // 23:40 CET + 30 minutes == 00:10 CET on the 16th, rounded up to 00:15 CET,
        // and the window of the 16th ends at 23:45 CET.
        bounds_test(
            &auction,
            "2024-01-15T22:40:00Z".parse().unwrap(),
            Some((
                "2024-01-15T23:15:00Z".parse().unwrap(),
                "2024-01-16T22:45:00Z".parse().unwrap(),
            )),
        );
    }
    #[test]
    fn continuous_same_day_only() {
        let auction =
            ContinuousAuctionTemporality::try_new(15, 0, 2.hours(), "CET").unwrap();
        // 10:00 CET is before the 15:00 CET next day gate opening.
        bounds_test(
            &auction,
            "2024-01-15T09:00:00Z".parse().unwrap(),
            Some((
                "2024-01-15T11:00:00Z".parse().unwrap(),
                "2024-01-15T22:45:00Z".parse().unwrap(),
            )),
        );
    }
    #[test]
    fn continuous_next_only() {
        let auction =
            ContinuousAuctionTemporality::try_new(15, 0, 2.hours(), "CET").unwrap();
        bounds_test(
            &auction,
            "2024-01-15T22:55:00Z".parse().unwrap(),
            Some((
                "2024-01-16T01:00:00Z".parse().unwrap(),
                "2024-01-16T22:45:00Z".parse().unwrap(),
            )),
        );
    }
    #[test]
    fn continuous_both_days() {
        let auction =
            ContinuousAuctionTemporality::try_new(15, 0, 2.hours(), "CET").unwrap();
        bounds_test(
            &auction,
            "2024-01-15T16:00:00Z".parse().unwrap(),
            Some((
                "2024-01-15T18:00:00Z".parse().unwrap(),
                "2024-01-16T22:45:00Z".parse().unwrap(),
            )),
        );
    }
}
// endregion: Tests
