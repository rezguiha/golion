use jiff::Timestamp;

use crate::units::power::KiloWatt;
#[derive(Debug)]
pub struct Commitment {
    pub start_at: Timestamp,
    pub input_power: KiloWatt,
    pub output_power: KiloWatt,
}
