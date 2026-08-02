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
    EndBeforeStart {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
    DatetimeNonMultipleOfStep {
        dt: DateTime<Utc>,
        step: MinuteGranularity,
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

fn check_datetime_multiple_step(
    dt: DateTime<Utc>,
    step: MinuteGranularity,
) -> Result<(), TimeGridError> {
    match dt.timestamp().rem_euclid(step.duration().num_seconds()) {
        0 => Ok(()),
        _ => Err(TimeGridError::DatetimeNonMultipleOfStep { dt, step }),
    }
}

impl RegularTimeGrid {
    pub fn try_new(
        start: DateTime<Utc>,
        step: MinuteGranularity,
        length: usize,
    ) -> crate::Result<Self> {
        check_datetime_multiple_step(start, step)?;
        let end = start + *step.duration() * (length as i32 - 1);
        check_datetime_multiple_step(end, step)?;
        Ok(RegularTimeGrid { start, step, length, end })
    }
    pub fn try_new_start_end(
        start: DateTime<Utc>,
        step: MinuteGranularity,
        end: DateTime<Utc>,
    ) -> crate::Result<Self> {
        if end <= start {
            return Err(TimeGridError::EndBeforeStart { start, end }.into());
        }
        check_datetime_multiple_step(start, step)?;
        check_datetime_multiple_step(end, step)?;

        let length: usize = (end.timestamp() - start.timestamp())
            .div_euclid(step.duration().num_seconds())
            .try_into()
            .map_err(TimeGridError::from)?;

        Ok(RegularTimeGrid { start, step, length, end })
    }

    pub fn index_of(&self, dt: &DateTime<Utc>) -> crate::Result<usize> {
        if (*dt < self.start) | (*dt > self.end) {
            return Err(TimeGridError::OutsideBounds {
                dt: *dt,
                start: self.start,
                end: self.end,
            }
            .into());
        }
        check_datetime_multiple_step(*dt, self.step)?;
        match usize::try_from(
            (*dt - self.start).num_seconds() / self.step.duration().num_seconds(),
        ) {
            Ok(index) => Ok(index),
            Err(_) => {
                Err(TimeGridError::MissAlignedDatetime { dt: *dt, step: self.step }
                    .into())
            }
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = DateTime<Utc>> {
        (0..=self.length).map(|i| self.start + *self.step.duration() * i as i32)
    }
}

// endregion: Regular Grid Struct and Traits
