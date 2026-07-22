/// Granularity in minutes struct represents the possible
/// allowed values of steps to have in timeseries.
use crate::serde_type;
use chrono::Duration;
use derive_more::{Display, From};
// region: Time Granularity
serde_type! {
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct MinuteGranularity(Duration);
// Made the Duration attribute private to force
// during use the usage of the try_from method
// to validate values and omit other ways of creation
// that may bypass it.
}
impl MinuteGranularity {
    pub fn duration(&self) -> &Duration {
        &self.0
    }
}
#[derive(Debug, Display, From)]
pub struct InvalidGranularity {
    #[display("invalid granularity: {granularity:?} (must be 15, 30, or 60 minutes)")]
    granularity: Duration,
}

impl TryFrom<Duration> for MinuteGranularity {
    type Error = InvalidGranularity;
    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        [Duration::minutes(15), Duration::minutes(30), Duration::minutes(60)]
            .contains(&value)
            .then_some(MinuteGranularity(value))
            .ok_or(InvalidGranularity { granularity: value })
    }
}
// endregion: Time Granularity
