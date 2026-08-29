use jiff::ToSpan;

use crate::countries::Countries;
use crate::market::temporality::{
    ContinuousAuctionTemporality, StaticAuctionTemporality, TimeDefinedInterval,
};
#[derive(Debug)]
pub struct SpotDayAheadConfig {
    pub country: Countries,
    pub auction: StaticAuctionTemporality,
}

impl SpotDayAheadConfig {
    pub fn try_new(country: Countries) -> crate::Result<Self> {
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
        };
        Ok(Self { country, auction })
    }
}

#[derive(Debug)]
pub struct IntraDayContinuousConfig {
    pub country: Countries,
    pub auction: ContinuousAuctionTemporality,
}

impl IntraDayContinuousConfig {
    pub fn try_new(country: Countries) -> crate::Result<Self> {
        let auction = match country {
            Countries::FR | Countries::ES | Countries::PT | Countries::IT => {
                ContinuousAuctionTemporality::try_new(15, 0, 1.hours(), "CET")?
            }
            Countries::DE => {
                ContinuousAuctionTemporality::try_new(15, 0, 15.minutes(), "CET")?
            }
            Countries::BE => {
                ContinuousAuctionTemporality::try_new(14, 0, 1.hours(), "CET")?
            }
        };
        Ok(Self { country, auction })
    }
}

#[derive(Debug)]
pub struct IntradayAuction1Config {
    pub country: Countries,
    pub auction: StaticAuctionTemporality,
}
impl IntradayAuction1Config {
    pub fn try_new(country: Countries) -> crate::Result<Self> {
        let harmonized_sidc_ida1 = StaticAuctionTemporality::try_new(
            TimeDefinedInterval::try_new(0, 0, 15, 0, 0.days())?,
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
            | Countries::PT => harmonized_sidc_ida1,
        };
        Ok(Self { country, auction })
    }
}

#[derive(Debug)]
pub struct IntradayAuction2Config {
    pub country: Countries,
    pub auction: StaticAuctionTemporality,
}
impl IntradayAuction2Config {
    pub fn try_new(country: Countries) -> crate::Result<Self> {
        let harmonized_sidc_ida2 = StaticAuctionTemporality::try_new(
            TimeDefinedInterval::try_new(0, 0, 22, 0, 0.days())?,
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
            | Countries::PT => harmonized_sidc_ida2,
        };
        Ok(Self { country, auction })
    }
}

#[derive(Debug)]
pub struct IntradayAuction3Config {
    pub country: Countries,
    pub auction: StaticAuctionTemporality,
}

impl IntradayAuction3Config {
    /// IDA3 is harmonised across SIDC: gate opens on D-1 and closes at 10:00
    /// CET on delivery day D, clearing only the second half of the day,
    /// D [12:00, 24:00).
    pub fn try_new(country: Countries) -> crate::Result<Self> {
        let harmonized_sidc_ida3 = StaticAuctionTemporality::try_new(
            TimeDefinedInterval::try_new(0, 0, 10, 0, 1.days())?,
            TimeDefinedInterval::try_new(12, 0, 23, 59, 0.days())?,
            0.days(),
            "CET",
        )?;
        let auction = match country {
            Countries::BE
            | Countries::DE
            | Countries::IT
            | Countries::FR
            | Countries::ES
            | Countries::PT => harmonized_sidc_ida3,
        };
        Ok(Self { country, auction })
    }
}
