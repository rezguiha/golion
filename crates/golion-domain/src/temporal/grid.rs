use super::step::{MinuteStep, MinuteStepError};
/// Regular time grid definition that encapsulates
/// the time related logic that enables us to have
/// a computation of the index a particular row
/// without the need of using a hashmap or binary
/// search on a timestamp index.
use derive_more::From;
use jiff::Timestamp;

// region: Regular Grid Errors

#[derive(Debug, From)]
pub enum TimeGridError {
    OutsideBounds {
        dt: Timestamp,
        start: Timestamp,
        end: Timestamp,
    },
    MissAlignedDatetime {
        dt: Timestamp,
        step: MinuteStep,
    },
    EndBeforeStart {
        start: Timestamp,
        end: Timestamp,
    },
    #[from]
    StepError(MinuteStepError),
    #[from]
    LengthOverFlow(std::num::TryFromIntError),
}
// endregion: Grid Errors

// region: Regular Grid Struct and Traits
/// Continuous time grid definition.
#[derive(Debug)]
pub struct RegularTimeGrid {
    /// First timestamp of timeseries.
    pub start: Timestamp,
    /// Granularity of timeseries.
    pub step: MinuteStep,
    /// Length of timeseries.
    pub length: usize,
    /// Computed end of timeseries.
    pub end: Timestamp,
}

impl RegularTimeGrid {
    pub fn try_new(
        start: Timestamp,
        step: MinuteStep,
        length: usize,
    ) -> crate::Result<Self> {
        step.check_datetime_multiple_step(start)?;
        let end = start + *step.duration() * (length as i32 - 1);
        step.check_datetime_multiple_step(end)?;
        Ok(RegularTimeGrid { start, step, length, end })
    }
    pub fn try_new_start_end(
        start: Timestamp,
        step: MinuteStep,
        end: Timestamp,
    ) -> crate::Result<Self> {
        if end <= start {
            return Err(TimeGridError::EndBeforeStart { start, end }.into());
        }
        step.check_datetime_multiple_step(start)?;
        step.check_datetime_multiple_step(end)?;

        let length: usize = (end.duration_since(start))
            .as_secs()
            .div_euclid(step.duration().as_secs())
            .try_into()
            .map_err(TimeGridError::from)?;

        Ok(RegularTimeGrid { start, step, length, end })
    }

    pub fn index_of(&self, dt: &Timestamp) -> crate::Result<usize> {
        if (*dt < self.start) | (*dt > self.end) {
            return Err(TimeGridError::OutsideBounds {
                dt: *dt,
                start: self.start,
                end: self.end,
            }
            .into());
        }
        self.step.check_datetime_multiple_step(*dt)?;
        match usize::try_from(
            (dt.duration_since(self.start)).as_secs() / self.step.duration().as_secs(),
        ) {
            Ok(index) => Ok(index),
            Err(_) => {
                Err(TimeGridError::MissAlignedDatetime { dt: *dt, step: self.step }
                    .into())
            }
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = Timestamp> {
        self.start.series(self.step.span()).take_while(|dt| dt <= &self.end)
    }
}

// endregion: Regular Grid Struct and Traits
