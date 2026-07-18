use super::grid::RegularTimeGrid;
use crate::{Error, Result};
use chrono::{DateTime, Utc};
use derive_more::From;
pub trait TimeStampedUtc {
    fn start_at(&self) -> &DateTime<Utc>;
}

#[derive(Debug)]
pub struct TimeSeries<T> {
    pub grid: RegularTimeGrid,
    pub data: Vec<T>,
}
#[derive(Debug, From)]
pub enum TimeSeriesError {
    TooShort { length: usize, minimal: i32 },
    GridEndMismatch { expected: DateTime<Utc>, actual: DateTime<Utc> },
    MissingValueAtTime { datetime: DateTime<Utc> },
}
impl<T> TimeSeries<T> {
    pub fn at(&self, dt: &DateTime<Utc>) -> Result<&T> {
        let index = self.grid.index_of(dt)?;
        self.data
            .get(index)
            .ok_or(TimeSeriesError::MissingValueAtTime { datetime: *dt }.into())
    }
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
        let step = *second - *start;
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
