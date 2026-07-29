use super::step::MinuteGranularity;
/// Regular time grid definition that encapsulates
/// the time related logic that enables us to have
/// a computation of the index a particular row
/// without the need of using a hashmap or binary
/// search on a timestamp index.
use chrono::{DateTime, Utc};
use derive_more::From;
// region: Regular Grid Errors

#[derive(Debug, From)]
pub enum TimeGridError {
    OutsideBounds {
        dt: DateTime<Utc>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
    MissAlignedDatetime {
        dt: DateTime<Utc>,
        step: MinuteGranularity,
    },

    EndDateTimeOverFlow {
        start: DateTime<Utc>,
        step: MinuteGranularity,
        length: i32,
    },
    #[from]
    LengthOverFlow(std::num::TryFromIntError),
}
// endregion: Grid Errors

// region: Regular Grid Struct and Traits
/// Continuous time grid definition.
#[derive(Debug)]
pub struct RegularTimeGrid {
    /// First timestamp of timeseries.
    pub start: DateTime<Utc>,
    /// Granularity of timeseries.
    pub step: MinuteGranularity,
    /// Length of timeseries.
    pub length: usize,
    /// Computed end of timeseries.
    pub end: DateTime<Utc>,
}
impl RegularTimeGrid {
    pub fn try_new(
        start: DateTime<Utc>,
        step: MinuteGranularity,
        length: usize,
    ) -> Result<Self, TimeGridError> {
        let length_int = i32::try_from(length)?;
        let end =
            start.checked_add_signed(*step.duration() * (length_int - 1)).ok_or({
                TimeGridError::EndDateTimeOverFlow { start, step, length: length_int }
            })?;
        Ok(RegularTimeGrid { start, step, length, end })
    }

    pub fn index_of(&self, dt: &DateTime<Utc>) -> Result<usize, TimeGridError> {
        if (*dt < self.start) | (*dt > self.end) {
            return Err(TimeGridError::OutsideBounds {
                dt: *dt,
                start: self.start,
                end: self.end,
            });
        }
        match usize::try_from(
            (*dt - self.start).num_seconds() / self.step.duration().num_seconds(),
        ) {
            Ok(index) => Ok(index),
            Err(_) => {
                Err(TimeGridError::MissAlignedDatetime { dt: *dt, step: self.step })
            }
        }
    }
}
// endregion: Regular Grid Struct and Traits
