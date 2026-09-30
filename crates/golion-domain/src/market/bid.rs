/// Bid Minimal Defining Characteristics
use crate::temporal::series::TimeStampedUtc;
use crate::temporal::step::MinuteStep;
use crate::units::power::{KiloWatt, KiloWattHour};
use derive_more::{Add, From, Into};
use jiff::{Span, Timestamp};
/// Represents market power increments.
/// For example:
///     -ancillary markets : 1000 kW
///     -wholesale markets: 100 kW
#[derive(PartialEq, From, Add, Into, Debug, Clone, Copy, Hash, Eq)]
pub struct KiloWattIncrement(u16);
impl KiloWattIncrement {
    pub fn value(&self) -> u16 {
        self.0
    }
}

/// Represents market product specifications.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ProductSpecifications {
    pub step: MinuteStep,
    pub increment: KiloWattIncrement,
}

impl ProductSpecifications {
    pub fn try_new(step: Span, increment_kw: u16) -> crate::Result<Self> {
        Ok(Self { step: step.try_into()?, increment: KiloWattIncrement(increment_kw) })
    }
}

#[derive(Debug)]
pub struct AncillaryBid {
    pub start_at: Timestamp,
    pub downward_power: KiloWatt,
    pub upward_power: KiloWatt,
}

#[derive(Debug)]
pub struct WholesaleBid {
    pub start_at: Timestamp,
    pub sell_energy: KiloWattHour,
    pub buy_energy: KiloWattHour,
}

impl TimeStampedUtc for WholesaleBid {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}
impl TimeStampedUtc for AncillaryBid {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}
