/// Bid Minimal Defining Characteristics
use crate::temporal::step::MinuteStep;
use derive_more::{Add, From, Into};

/// Represents market power increments.
/// For example:
///     -ancillary markets : 1000 kW
///     -wholesale markets: 100 kW
#[derive(PartialEq, From, Add, Into, Debug, Clone, Copy)]
pub struct KiloWattIncrement(u16);

#[derive(Debug)]
pub struct BidSpecs {
    pub step: MinuteStep,
    pub increment: KiloWattIncrement,
}
