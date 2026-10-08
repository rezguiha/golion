/// Projected Physical state of asset.
/// Represents the information we want to track
/// on a particular asset.
use jiff::Timestamp;

use crate::units::power::{KiloWatt, KiloWattHour};

#[derive(Debug)]
pub struct BessState {
    pub start_at: Timestamp,
    pub dispatch: KiloWatt,
    pub soc: KiloWattHour,
}

#[derive(Debug)]
pub struct OtherAssetState {
    pub start_at: Timestamp,
    pub dispatch: KiloWatt,
}
