/// Step in minutes struct represents the possible
/// allowed values of steps to have in timeseries.
use chrono::{DateTime, Duration, Utc};
use derive_more::{Display, From};
// region: MinuteStep

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct MinuteStep(Duration);
// Made the Duration attribute private to force
// during use the usage of the try_from method
// to validate values and omit other ways of creation
// that may bypass it.

impl MinuteStep {
    pub fn duration(&self) -> &Duration {
        &self.0
    }
    pub fn check_datetime_multiple_step(&self, dt: DateTime<Utc>) -> crate::Result<()> {
        match dt.timestamp().rem_euclid(self.duration().num_seconds()) {
            0 => Ok(()),
            _ => Err(MinuteStepError::DatetimeNonMultipleOfStep {
                dt,
                step: *self.duration(),
            }
            .into()),
        }
    }
}
#[derive(Debug, Display, From)]
pub enum MinuteStepError {
    #[display("invalid step: {step:?} (must be 15, 30, or 60 minutes)")]
    InvalidGranularity { step: Duration },
    #[display("Datetime {dt} must be multiple of {step}.")]
    DatetimeNonMultipleOfStep { dt: DateTime<Utc>, step: Duration },
}

impl TryFrom<Duration> for MinuteStep {
    type Error = crate::Error;
    fn try_from(value: Duration) -> crate::Result<Self> {
        [Duration::minutes(15), Duration::minutes(30), Duration::minutes(60)]
            .contains(&value)
            .then_some(MinuteStep(value))
            .ok_or(MinuteStepError::InvalidGranularity { step: value }.into())
    }
}
// endregion: MinuteStep
