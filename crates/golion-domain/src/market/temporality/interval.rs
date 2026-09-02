use jiff::civil::Time;
use jiff::{Span, ToSpan};

use crate::market::error::MarketError;

/// This struct is used to represent the time separating auction closure and bidding
/// start. For example : for day ahead market the bidding is by definition on the next day
/// from auction closure.
/// It is also used to represent the time separating auction open and close times. For example
/// for intraday third auction.

#[derive(Debug)]
pub struct DeltaDays(Span);

impl DeltaDays {
    // Getter method.
    pub fn value(&self) -> &Span {
        &self.0
    }
}

impl TryFrom<Span> for DeltaDays {
    type Error = crate::Error;
    fn try_from(value: Span) -> crate::Result<Self> {
        [0.days().fieldwise(), 1.days().fieldwise()]
            .contains(&value.fieldwise())
            .then_some(DeltaDays(value))
            .ok_or(MarketError::InvalidDeltaDayValue { value }.into())
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
    pub fn try_new(
        open_hour: i8,
        open_minute: i8,
        close_hour: i8,
        close_minute: i8,
        delta_start_end: Span,
    ) -> crate::Result<Self> {
        let start_time = Time::new(open_hour, open_minute, 0, 0)?;
        let end_time = Time::new(close_hour, close_minute, 0, 0)?;
        let delta_start_end_days: DeltaDays = delta_start_end.try_into()?;
        if (start_time > end_time) & (delta_start_end_days.0.get_days() == 0) {
            Err(MarketError::InvalidTimeDefinedBound {
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
