/// Step struct represents the possible
/// allowed values of steps to have in timeseries.
use derive_more::{Display, From};

use jiff::{Error as JiffError, SignedDuration, Span, Timestamp};
use strum::{EnumIter, IntoEnumIterator};
// region: Step

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum Step {
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    FourHours,
}
impl Step {
    pub fn duration(&self) -> SignedDuration {
        match self {
            Self::FifteenMinutes => SignedDuration::from_mins(15),
            Self::ThirtyMinutes => SignedDuration::from_mins(30),
            Self::OneHour => SignedDuration::from_hours(1),
            Self::FourHours => SignedDuration::from_hours(4),
        }
    }
    pub fn span(&self) -> Span {
        match self {
            Self::FifteenMinutes => Span::new().minutes(15),
            Self::ThirtyMinutes => Span::new().minutes(30),
            Self::OneHour => Span::new().hours(1),
            Self::FourHours => Span::new().hours(4),
        }
    }

    pub fn total_hours_f64(&self) -> f64 {
        self.duration().as_secs_f64() / 3600.0_f64
    }
    pub fn check_datetime_multiple_step(&self, dt: Timestamp) -> crate::Result<()> {
        match dt.as_second().rem_euclid(self.duration().as_secs()) {
            0 => Ok(()),
            _ => {
                Err(StepError::DatetimeNonMultipleOfStep { dt, step: self.duration() }
                    .into())
            }
        }
    }
}

#[derive(Debug, Display, From)]
pub enum StepError {
    #[display("invalid step: {step:?} ( possible values : {possible_values:?})")]
    InvalidGranularity { step: SignedDuration, possible_values: Vec<SignedDuration> },
    #[display("Datetime {dt} must be multiple of {step}.")]
    DatetimeNonMultipleOfStep { dt: Timestamp, step: SignedDuration },
    #[from]
    JiffConversionError(JiffError),
}

// endregion: Step

// region: Conversions
impl TryFrom<SignedDuration> for Step {
    type Error = crate::Error;
    fn try_from(value: SignedDuration) -> crate::Result<Self> {
        for possible_step in Self::iter() {
            if possible_step.duration() == value {
                return Ok(possible_step);
            }
        }
        Err(StepError::InvalidGranularity {
            step: value,
            possible_values: Self::iter().map(|step| step.duration()).collect(),
        }
        .into())
    }
}
impl TryFrom<Span> for Step {
    type Error = crate::Error;
    fn try_from(value: Span) -> crate::Result<Self> {
        let value: SignedDuration =
            value.try_into().map_err(StepError::JiffConversionError)?;
        value.try_into()
    }
}
// endregion: Conversions
