use jiff::ToSpan;

use crate::countries::Countries;
use crate::market::temporality::{
    ContinuousAuctionTemporality, StaticAuctionTemporality, TimeDefinedInterval,
};
#[derive(Debug)]
pub struct MarketConfig<T> {
    pub country: Countries,
    pub auction: T,
}
pub type SpotDayAheadConfig = MarketConfig<StaticAuctionTemporality>;

impl SpotDayAheadConfig {
    pub fn new(country: Countries) -> crate::Result<Self> {
        let common_european_temporality = StaticAuctionTemporality::try_new(
            TimeDefinedInterval::try_new(0, 0, 12, 0, 1.days())?,
            TimeDefinedInterval::try_new(0, 0, 23, 59, 0.days())?,
            1.days(),
            "CET",
        )?;
        let auction = match country {
            Countries::BE
            | Countries::DE
            | Countries::IT
            | Countries::FR
            | Countries::ES
            | Countries::PT => common_european_temporality,
            Countries::CH => StaticAuctionTemporality::try_new(
                TimeDefinedInterval::try_new(0, 0, 11, 0, 1.days())?,
                TimeDefinedInterval::try_new(0, 0, 23, 59, 0.days())?,
                1.days(),
                "CET",
            )?,
            Countries::GB => StaticAuctionTemporality::try_new(
                TimeDefinedInterval::try_new(0, 0, 10, 20, 1.days())?,
                TimeDefinedInterval::try_new(0, 0, 23, 59, 0.days())?,
                1.days(),
                "CET",
            )?,
        };
        Ok(Self { country, auction })
    }
}

pub type IntraDayContinuousConfig = MarketConfig<ContinuousAuctionTemporality>;

impl IntraDayContinuousConfig {
    pub fn try_new(country: Countries) -> crate::Result<Self> {
        let auction = match country {
            Countries::FR
            | Countries::CH
            | Countries::ES
            | Countries::PT
            | Countries::IT => {
                ContinuousAuctionTemporality::try_new(15, 0, 1.hours(), "CET")?
            }
            Countries::DE => {
                ContinuousAuctionTemporality::try_new(15, 0, 15.minutes(), "CET")?
            }
            Countries::BE => {
                ContinuousAuctionTemporality::try_new(14, 0, 1.hours(), "CET")?
            }
            Countries::GB => {
                ContinuousAuctionTemporality::try_new(0, 0, 1.hours(), "GMT")?
            }
        };
        Ok(Self { country, auction })
    }
}
