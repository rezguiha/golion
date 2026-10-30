/// Bid Minimal Defining Characteristics
use crate::temporal::series::TimeStampedUtc;
use crate::temporal::step::Step;
use crate::units::power::{KiloWatt, KiloWattHour};
use derive_more::{Add, From, Into};
use jiff::{
    RoundMode, SignedDuration, Span, Timestamp, TimestampRound, ToSpan, Unit, Zoned,
    ZonedRound,
};
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
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Symmetry {
    Symmetric,
    Asymmetric,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct ProductSpecifications {
    pub step: Step,
    pub increment: KiloWattIncrement,
    pub symmetry: Symmetry,
}

impl ProductSpecifications {
    pub fn try_new(
        step: Span,
        increment_kw: u16,
        symmetry: Symmetry,
    ) -> crate::Result<Self> {
        Ok(Self {
            step: step.try_into()?,
            increment: KiloWattIncrement(increment_kw),
            symmetry,
        })
    }
}
impl ProductSpecifications {
    /// Computes next product according to a reference.
    ///
    /// For products less or equal to 1 hour the DST has no influence on them.
    ///
    /// For products over 1 hour like 4 hours (FCR) where the product crosses the boundary
    /// the rules make variable length products where we may have either 3 hours or 5 hours
    /// of duration of product depending on dst transition (winter or summer) but the local
    /// time intervals remain the same [0,4,8,12,16..].
    /// Thus our use of ZonedRound for those
    /// products and Timestamp round for the rest (<= 1 hour).
    pub fn next_product(&self, reference: &Zoned) -> crate::Result<(Zoned, Zoned)> {
        let duration = self.step.duration();
        let round_up = |dt: &Zoned| -> crate::Result<Zoned> {
            if duration <= SignedDuration::from_hours(1) {
                let round = TimestampRound::new()
                    .smallest(Unit::Minute)
                    .increment(duration.as_mins())
                    .mode(RoundMode::Expand);
                Ok(dt.timestamp().round(round)?.to_zoned(dt.time_zone().clone()))
            } else {
                let round = ZonedRound::new()
                    .smallest(Unit::Hour)
                    .increment(duration.as_hours())
                    .mode(RoundMode::Expand);
                Ok(dt.round(round)?)
            }
        };
        let start = round_up(reference)?;
        let end = round_up(&start.checked_add(1.minute())?)?;
        Ok((start, end))
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
    fn start_at(&self) -> Timestamp {
        self.start_at
    }
}
impl TimeStampedUtc for AncillaryBid {
    fn start_at(&self) -> Timestamp {
        self.start_at
    }
}
