use chrono::{DateTime, Utc};
use thiserror::Error;
#[derive(Debug, Error)]
pub enum SeriesError {
    #[error("No data found for at index : {index}")]
    MissingValueAtIndex { index: usize },
    #[error("No data found for at datetime : {datetime}")]
    MissingValueAtTime { datetime: DateTime<Utc> },
    #[error("{datetime} is misaligned with granularitY {granularity}")]
    NonAlignedValue { datetime: DateTime<Utc>, granularity: i64 },
    #[error("Series is misaligned with timeline [{start_timeline},{end_timeline}]")]
    MissAlignedWithTimeLine { start_timeline: DateTime<Utc>, end_timeline: DateTime<Utc> },
}
