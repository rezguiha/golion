use chrono::{DateTime, Duration, Utc};
use derive_more::From;

// region: Grid Errors

#[derive(Debug, From)]
pub enum TimeGridError {
    OutsideBounds {
        dt: DateTime<Utc>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
    MissAlignedDatetime {
        dt: DateTime<Utc>,
        step: Duration,
    },

    EndDateTimeOverFlow {
        start: DateTime<Utc>,
        step: Duration,
        length: i32,
    },
    #[from]
    LengthOverFlow(std::num::TryFromIntError),
}
// endregion: Grid Errors

// region: Regular Grid Struct and Traits
#[derive(Debug)]
pub struct RegularTimeGrid {
    pub start: DateTime<Utc>,
    pub step: Duration,
    pub length: usize,
    pub end: DateTime<Utc>,
}
impl RegularTimeGrid {
    pub fn try_new(
        start: DateTime<Utc>,
        step: Duration,
        length: usize,
    ) -> Result<Self, TimeGridError> {
        let length_int = i32::try_from(length)?;
        let end = start.checked_add_signed(step * length_int).ok_or({
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
        match usize::try_from((*dt - self.start).num_seconds() / self.step.num_seconds())
        {
            Ok(index) => Ok(index),
            Err(_) => {
                Err(TimeGridError::MissAlignedDatetime { dt: *dt, step: self.step })
            }
        }
    }
}
// endregion: Regular Grid Struct and Traits
