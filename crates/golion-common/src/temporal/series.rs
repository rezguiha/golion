use crate::error::SeriesError;
use chrono::{DateTime, Duration, Utc};

pub struct TimeSeries<T> {
    pub start_at: DateTime<Utc>,
    pub granularity: Duration,
    pub data: Vec<T>,
    _granularity_seconds: i64,
}

impl<T> TimeSeries<T> {
    pub fn new(start_at: DateTime<Utc>, granularity: Duration, data: Vec<T>) -> Self {
        let _granularity_seconds = granularity.num_seconds();
        Self { start_at, granularity, data, _granularity_seconds }
    }
    pub fn at_index(&self, i: usize) -> Result<&T, SeriesError> {
        self.data.get(i).ok_or(SeriesError::MissingValueAtIndex { index: i })
    }
    pub fn at(&self, dt: &DateTime<Utc>) -> Result<&T, SeriesError> {
        let delta_seconds = (*dt - self.start_at).num_seconds();
        let excess = delta_seconds.rem_euclid(self._granularity_seconds);
        if excess != 0 {
            return Err(SeriesError::NonAlignedValue {
                datetime: *dt,
                granularity: self._granularity_seconds,
            });
        }
        let stride = delta_seconds.div_euclid(self._granularity_seconds);

        match stride {
            s if s >= 0 => self.at_index(s as usize),
            _ => Err(SeriesError::MissingValueAtTime { datetime: *dt }),
        }
    }
}
