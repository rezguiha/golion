/// TimeSeries container composed of a regular grid
/// which encapsulates temporal information and
/// the data that it contains. This enables us
/// to have fast row access by index on an array of
/// struct shaped data.
use super::grid::RegularTimeGrid;
use super::step::MinuteGranularity;
use crate::{Error, Result};
use chrono::{DateTime, Utc};
use derive_more::From;
// region: TimeSeries Errors
#[derive(Debug, From)]
pub enum TimeSeriesError {
    TooShort { length: usize, minimal: i32 },
    GridEndMismatch { expected: DateTime<Utc>, actual: DateTime<Utc> },
    MissingValueAtTime { datetime: DateTime<Utc> },
}
// endregion: TimeSeries Errors

// region: TimeSeries Struct and Traits
#[derive(Debug)]
pub struct TimeSeries<T> {
    // Continuous time grid.
    pub grid: RegularTimeGrid,
    // Array of struct container.
    pub data: Vec<T>,
}

impl<T> TimeSeries<T> {
    pub fn at(&self, dt: &DateTime<Utc>) -> Result<&T> {
        let index = self.grid.index_of(dt)?;
        self.data
            .get(index)
            .ok_or(TimeSeriesError::MissingValueAtTime { datetime: *dt }.into())
    }
}
// endregion: TimeSeries Struct and Traits

// region: TimeSeries from Vector of data conversion trait.
/// Convenience trait to be able to use into TimeSeries directly
/// on a vector of data if the conversion is straight forward
/// with no extra transformation on the condition it has some temporal
/// information in each row it contains.
pub trait TimeStampedUtc {
    fn start_at(&self) -> &DateTime<Utc>;
}

impl<T: TimeStampedUtc> TryFrom<Vec<T>> for TimeSeries<T> {
    type Error = Error;
    fn try_from(data: Vec<T>) -> Result<TimeSeries<T>> {
        let length = data.len();
        if length < 2 {
            return Err(TimeSeriesError::TooShort { length, minimal: 2 }.into());
        };
        // We can use unwrap here sinc we tested that length of data is at least 2
        let start = data.first().unwrap().start_at();
        let second = data.get(1).unwrap().start_at();
        let step = MinuteGranularity::try_from(*second - *start)?;
        let end = data.last().unwrap().start_at();
        let grid = RegularTimeGrid::try_new(*start, step, length)?;

        match grid.end.eq(end) {
            true => Ok(Self { grid, data }),
            _ => {
                Err(TimeSeriesError::GridEndMismatch { expected: *end, actual: grid.end }
                    .into())
            }
        }
    }
}
// region: TimeSeries from Vector of data conversion trait.
