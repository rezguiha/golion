/// Bid Minimal Defining Characteristics
use crate::temporal::step::MinuteStep;
use derive_more::{Add, From, Into};
use jiff::Span;
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
