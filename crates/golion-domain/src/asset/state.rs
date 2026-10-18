/// Projected Physical state of asset.
/// Represents the information we want to track
/// on a particular asset.
use jiff::Timestamp;

use crate::units::power::{KiloWatt, KiloWattHour};

#[derive(Debug)]
pub struct BessState {
    pub start_at: Timestamp,
    // Dispatch Signal sent to asset. Positive for charge
    // negative for discharge.
    pub dispatch: KiloWatt,
    pub soc: KiloWattHour,
    // Upward ancillary commitments contribution provisined for this asset.
    pub ancillary_upward: KiloWatt,
    // Downward ancillary commitments contribution provisined for this asset.
    pub ancillary_downward: KiloWatt,
}

#[derive(Debug)]
pub struct OtherAssetState {
    pub start_at: Timestamp,
    pub dispatch: KiloWatt,
}
