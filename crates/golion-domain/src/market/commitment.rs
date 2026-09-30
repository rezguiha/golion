use jiff::Timestamp;

use crate::{
    temporal::{series::TimeStampedUtc, step::MinuteStep},
    units::power::{KiloWatt, KiloWattHour},
};
#[derive(Debug)]
pub struct PowerCommitment {
    pub start_at: Timestamp,
    pub input_power: KiloWatt,
    pub output_power: KiloWatt,
}

#[derive(Debug)]
pub struct EnergyCommitment {
    pub start_at: Timestamp,
    pub energy_net_position: KiloWattHour,
}
impl TimeStampedUtc for PowerCommitment {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}
impl TimeStampedUtc for EnergyCommitment {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}
impl EnergyCommitment {
    pub fn to_power_commitment(&self, step: &MinuteStep) -> PowerCommitment {
        let duration_seconds = step.duration().as_secs_f64();
        PowerCommitment {
            start_at: self.start_at,
            input_power: (self.energy_net_position.0.max(0.0) * 3600.0_f64
                / duration_seconds)
                .into(),
            output_power: (self.energy_net_position.0.min(0.0).abs() * 3600.0_f64
                / duration_seconds)
                .into(),
        }
    }
}
