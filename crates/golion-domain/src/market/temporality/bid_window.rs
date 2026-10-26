use jiff::civil::Time;
use jiff::tz::TimeZone;
use jiff::{Span, Timestamp, ToSpan, Zoned};

use crate::market::bid::ProductSpecifications;
use crate::market::error::MarketError;
use crate::market::temporality::auction::{
    ContinuousAuctionTemporality, DynamicAuctionTemporality, StaticAuctionTemporality,
};
use crate::temporal::grid::RegularTimeGrid;
use crate::temporal::step::Step;
// region: Bidding Time Boundaries and its trait implementations
#[derive(Debug)]
pub struct BidWindow {
    start: Timestamp,
    end: Timestamp,
}
impl BidWindow {
    pub fn try_new(start: Timestamp, end: Timestamp) -> crate::Result<Self> {
        if start >= end {
            Err(MarketError::BidWindowNotValid { start, end }.into())
        } else {
            Ok(Self { start, end })
        }
    }
    pub fn try_iter(
        &self,
        step: &Step,
    ) -> crate::Result<impl Iterator<Item = Timestamp>> {
        step.check_datetime_multiple_step(self.start)?;
        step.check_datetime_multiple_step(self.end)?;
        Ok(self.start.series(step.span()).take_while(|dt| *dt < self.end))
    }
}
/// Defines the ability to compute bidding start and end relative
/// to a reference timestamp which will be in our case the run time
/// of the process for each bid in the bidding horizon.
pub trait ToBidWindows {
    /// TimeZone of the market. Necessary to make link with local time
    /// definitions like for open and close times for bidding or auction times.
    fn timezone(&self) -> &TimeZone;
    /// Computes the available bidding horizon at reference time.
    /// This needs to return zoned datetimes because the market bid windows
    /// rely on local time cet.
    ///
    /// For example for FCR 4 hour bid. During DST a bid can be 3 hour or 5 hours.
    /// Also even for other markets like sport day ahead. Bidding bounds are on local
    /// time and during dst we may have 23 hour bidding or 25 hour bidding.
    fn bidding_horizon(
        &self,
        reference_time: &Timestamp,
    ) -> crate::Result<(Zoned, Zoned)>;

    fn to_bid_windows(
        &self,
        reference_time: &Timestamp,
        grid: &RegularTimeGrid,
        product_specifications: &ProductSpecifications,
    ) -> crate::Result<Vec<BidWindow>> {
        let bid_step = product_specifications.step;
        let grid_step = grid.step();
        if bid_step.duration().as_secs() % grid_step.duration().as_secs() != 0 {
            Err(MarketError::BidStepNotMultipleGridStep {
                bid_step: bid_step.duration(),
                grid_step: grid_step.duration(),
            }
            .into())
        } else {
            let mut bid_windows = Vec::<BidWindow>::new();
            // Get zoned start and end of bidding.
            let (start_bidding, end_bidding) = self.bidding_horizon(reference_time)?;

            // Make sure bid bounds are inside time index.
            let start_bidding =
                start_bidding.max(grid.start().to_zoned(self.timezone().clone()));
            let end_bidding =
                end_bidding.min(grid.end().to_zoned(self.timezone().clone()));
            // Compute first bid start by rounding up towards next multiple of bid.
            let (mut start_bid, mut end_bid) =
                product_specifications.next_product(&start_bidding)?;
            // Move by bid step from start of first bid to get full bids intervals
            while end_bid <= end_bidding {
                bid_windows.push(BidWindow::try_new(
                    start_bid.timestamp(),
                    end_bid.timestamp(),
                )?);
                (start_bid, end_bid) = product_specifications.next_product(&end_bid)?;
            }
            Ok(bid_windows)
        }
    }
}

/// Convenience method that adds a delta in days and sets the time to the new
/// zoned datetime.
fn add_and_set_time(reference: &Zoned, delta: &Span, time: Time) -> crate::Result<Zoned> {
    let new = reference.checked_add(delta).and_then(|dt| dt.with().time(time).build())?;
    Ok(new)
}

impl ToBidWindows for StaticAuctionTemporality {
    fn timezone(&self) -> &TimeZone {
        &self.timezone
    }
    fn bidding_horizon(
        &self,
        reference_time: &Timestamp,
    ) -> crate::Result<(Zoned, Zoned)> {
        // We are using clone here on timezone as it is cheap to clone
        // and to_zoned requires to pass ownership of timezone.
        let zoned_reference_time = reference_time.to_zoned(self.timezone().clone());
        let zoned_open_at =
            zoned_reference_time.with().time(self.open_interval.start_time).build()?;
        let zoned_close_at = add_and_set_time(
            &zoned_open_at,
            self.open_interval.delta_start_end.value(),
            self.open_interval.end_time,
        )?;
        // Past closure time return past and equal bound window that makes
        // generating bid impossible.
        if zoned_close_at <= zoned_reference_time {
            Ok((zoned_reference_time.yesterday()?, zoned_reference_time.yesterday()?))
        } else {
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
            Ok((zoned_bidding_start, zoned_bidding_end))
        }
    }
}

impl ToBidWindows for DynamicAuctionTemporality {
    fn timezone(&self) -> &TimeZone {
        &self.timezone
    }
    fn bidding_horizon(
        &self,
        reference_time: &Timestamp,
    ) -> crate::Result<(Zoned, Zoned)> {
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
        Ok((zoned_bidding_start, zoned_bidding_end))
    }
}

impl ToBidWindows for ContinuousAuctionTemporality {
    fn timezone(&self) -> &TimeZone {
        &self.timezone
    }
    /// Computes bidding window for a continuous auction temporality like intraday
    /// continuous market for example.
    ///
    /// If bidding start is in same day as reference available bids starts there
    /// and finish at the end of the same day or next day depending on whether the
    /// next day gate is open.
    ///
    /// If neutralization delay pushes the start to the day after the day of reference,
    /// if the gate is open start from it until end of the day.
    ///
    /// Otherwise, bidding start being in the day after end of bidding makes it
    /// impossible to generate windows which is the expected behavior as no bid is
    /// available in that situation.
    fn bidding_horizon(
        &self,
        reference_time: &Timestamp,
    ) -> crate::Result<(Zoned, Zoned)> {
        let zoned_reference_time = reference_time.to_zoned(self.timezone().clone());
        // Add neutralization delay to get bidding start time.
        let bidding_start =
            zoned_reference_time.checked_add(self.neutralization_delay)?;

        // Compute gate open time
        let next_day_gate_open_at =
            zoned_reference_time.with().time(self.next_day_gate_open_time).build()?;
        // Compute bidding end depending on gate open time (end of this day or next day)
        let bidding_end = match zoned_reference_time {
            reference if reference >= next_day_gate_open_at => {
                reference.saturating_add(2.days()).start_of_day()?
            }
            reference => reference.tomorrow()?.start_of_day()?,
        };
        Ok((bidding_start, bidding_end))
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
        bid_window::ToBidWindows,
        interval::TimeDefinedInterval,
    };
    use crate::temporal::step::Step;

    fn product_specifications(step_minutes: i64) -> ProductSpecifications {
        ProductSpecifications {
            step: Step::try_from(SignedDuration::from_mins(step_minutes)).unwrap(),
            increment: KiloWattIncrement::from(100),
        }
    }

    /// Checks the bidding horizon of an auction at a reference time.
    fn bidding_horizon_test(
        auction: &impl ToBidWindows,
        reference_time: Timestamp,
        expected_window: (Timestamp, Timestamp),
    ) {
        let actual_window = auction
            .bidding_horizon(&reference_time)
            .map(|(start, end)| (start.timestamp(), end.timestamp()))
            .unwrap();
        println!("Actual : {:?}, expected: {:?}", actual_window, expected_window);
        assert_eq!(actual_window, expected_window);
    }
    #[test]
    fn dynamic_short_interval() {
        let auction = DynamicAuctionTemporality::try_new(
            TimeDefinedInterval::try_new(10, 0, 10, 3, 0.days()).unwrap(),
            0.minutes(),
            "CET",
        )
        .unwrap();
        // 10:00 CET, the window is the whole 10:00-10:03 CET interval.
        bidding_horizon_test(
            &auction,
            "2024-01-15T09:00:00Z".parse().unwrap(),
            (
                "2024-01-15T09:00:00Z".parse().unwrap(),
                "2024-01-15T09:03:00Z".parse().unwrap(),
            ),
        );
    }
    #[test]
    fn dynamic_reference_inside_interval() {
        let auction = DynamicAuctionTemporality::try_new(
            TimeDefinedInterval::try_new(9, 50, 10, 20, 0.days()).unwrap(),
            0.minutes(),
            "CET",
        )
        .unwrap();
        // 10:00 CET is inside 9:50-10:20 CET, so the window opens at the reference.
        bidding_horizon_test(
            &auction,
            "2024-01-15T09:00:00Z".parse().unwrap(),
            (
                "2024-01-15T09:00:00Z".parse().unwrap(),
                "2024-01-15T09:20:00Z".parse().unwrap(),
            ),
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
        // 10:00 CET + 30 minutes == 10:30 CET, until 23:59 CET.
        bidding_horizon_test(
            &auction,
            "2024-01-15T09:00:00Z".parse().unwrap(),
            (
                "2024-01-15T09:30:00Z".parse().unwrap(),
                "2024-01-15T22:59:00Z".parse().unwrap(),
            ),
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
        // 23:40 CET + 30 minutes == 00:10 CET on the 16th, and the window of the
        // 16th ends at 23:59 CET.
        bidding_horizon_test(
            &auction,
            "2024-01-15T22:40:00Z".parse().unwrap(),
            (
                "2024-01-15T23:10:00Z".parse().unwrap(),
                "2024-01-16T22:59:00Z".parse().unwrap(),
            ),
        );
    }
    #[test]
    fn continuous_same_day_only() {
        let auction =
            ContinuousAuctionTemporality::try_new(15, 0, 2.hours(), "CET").unwrap();
        // 10:00 CET is before the 15:00 CET next day gate opening: 12:00 CET
        // until the end of the day.
        bidding_horizon_test(
            &auction,
            "2024-01-15T09:00:00Z".parse().unwrap(),
            (
                "2024-01-15T11:00:00Z".parse().unwrap(),
                "2024-01-15T23:00:00Z".parse().unwrap(),
            ),
        );
    }
    #[test]
    fn continuous_next_only() {
        let auction =
            ContinuousAuctionTemporality::try_new(15, 0, 2.hours(), "CET").unwrap();
        // 23:55 CET + 2 hours == 01:55 CET on the 16th, after the gate opening:
        // until the end of the 16th.
        bidding_horizon_test(
            &auction,
            "2024-01-15T22:55:00Z".parse().unwrap(),
            (
                "2024-01-16T00:55:00Z".parse().unwrap(),
                "2024-01-16T23:00:00Z".parse().unwrap(),
            ),
        );
    }
    #[test]
    fn continuous_both_days() {
        let auction =
            ContinuousAuctionTemporality::try_new(15, 0, 2.hours(), "CET").unwrap();
        // 17:00 CET is after the gate opening: 19:00 CET until the end of the 16th.
        bidding_horizon_test(
            &auction,
            "2024-01-15T16:00:00Z".parse().unwrap(),
            (
                "2024-01-15T18:00:00Z".parse().unwrap(),
                "2024-01-16T23:00:00Z".parse().unwrap(),
            ),
        );
    }
}
// endregion: Tests
