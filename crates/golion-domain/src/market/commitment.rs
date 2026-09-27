use jiff::Timestamp;

use crate::{
    temporal::{series::TimeStampedUtc, step::MinuteStep},
    units::power::{KiloWatt, KiloWattHour},
};
#[derive(Debug)]
pub struct Commitment<U> {
    pub start_at: Timestamp,
    pub input: U,
    pub output: U,
}

pub type PowerCommitment = Commitment<KiloWatt>;
pub type EnergyCommitment = Commitment<KiloWattHour>;

impl<U> TimeStampedUtc for Commitment<U> {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}
impl EnergyCommitment {
    pub fn to_power_commitment(&self, step: &MinuteStep) -> PowerCommitment {
        let duration_seconds = step.duration().as_secs_f64();
        PowerCommitment {
            start_at: self.start_at,
            input: (self.input.0 * 3600.0_f64 / duration_seconds).into(),
            output: (self.output.0 * 3600.0_f64 / duration_seconds).into(),
        }
    }
}
