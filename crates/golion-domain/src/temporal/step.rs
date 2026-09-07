use derive_more::{Display, From};
/// Step in minutes struct represents the possible
/// allowed values of steps to have in timeseries.
use jiff::{Error as JiffError, SignedDuration, Span, Timestamp};
// region: MinuteStep

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct MinuteStep(SignedDuration);
// Made the SignedDuration attribute private to force
// during use the usage of the try_from method
// to validate values and omit other ways of creation
// that may bypass it.

impl MinuteStep {
    pub fn duration(&self) -> &SignedDuration {
        &self.0
    }
    pub fn span(&self) -> Span {
        Span::new().seconds(self.0.as_secs())
    }
    pub fn check_datetime_multiple_step(&self, dt: Timestamp) -> crate::Result<()> {
        match dt.as_second().rem_euclid(self.duration().as_secs()) {
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
    InvalidGranularity { step: SignedDuration },
    #[display("Datetime {dt} must be multiple of {step}.")]
    DatetimeNonMultipleOfStep { dt: Timestamp, step: SignedDuration },
    #[from]
    JiffConversionError(JiffError),
}

impl TryFrom<SignedDuration> for MinuteStep {
    type Error = crate::Error;
    fn try_from(value: SignedDuration) -> crate::Result<Self> {
        [
            SignedDuration::from_mins(15),
            SignedDuration::from_mins(30),
            SignedDuration::from_mins(60),
        ]
        .contains(&value)
        .then_some(MinuteStep(value))
        .ok_or(MinuteStepError::InvalidGranularity { step: value }.into())
    }
}
impl TryFrom<Span> for MinuteStep {
    type Error = crate::Error;
    fn try_from(value: Span) -> crate::Result<Self> {
        let value: SignedDuration =
            value.try_into().map_err(MinuteStepError::JiffConversionError)?;
        value.try_into()
    }
}

// endregion: MinuteStep
