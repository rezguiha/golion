use jiff::ToSpan;

use crate::countries::Countries;
use crate::market::error::MarketError;
use crate::market::market_type::{
    AncillaryMarketType, CapacityAncillaryMarketType, EnergyAncillaryMarketType,
    MarketType, WholesaleMarketType,
};
use crate::market::temporality::{
    auction::{
        ContinuousAuctionTemporality, DynamicAuctionTemporality, StaticAuctionTemporality,
    },
    bid_time_bounds::ToBidTimeBounds,
    interval::TimeDefinedInterval,
};

// region: Market config
/// A particular market configuration
#[derive(Debug)]
pub struct MarketConfig<T: ToBidTimeBounds> {
    /// Country of market.
    pub country: Countries,
    /// Auction temporality
    pub auction: T,
}
/// All market configuration representation.
#[derive(Debug)]
pub enum AllMarketConfig {
    SpotDayAhead(MarketConfig<StaticAuctionTemporality>),
    IntradayAuction1(MarketConfig<StaticAuctionTemporality>),
    IntradayAuction2(MarketConfig<StaticAuctionTemporality>),
    IntradayAuction3(MarketConfig<StaticAuctionTemporality>),
    IntradayContinuous(MarketConfig<ContinuousAuctionTemporality>),
    AfrrFree(MarketConfig<DynamicAuctionTemporality>),
    Afrr(MarketConfig<DynamicAuctionTemporality>),
    Fcr(MarketConfig<DynamicAuctionTemporality>),
}

impl AllMarketConfig {
    pub fn try_new(market_type: &MarketType, country: &Countries) -> crate::Result<Self> {
        match market_type {
            MarketType::WholeSale(WholesaleMarketType::SpotDayAhead) => {
                Self::spot_day_ahead(country)
            }
            MarketType::WholeSale(WholesaleMarketType::IntradayAuction1) => {
                Self::intraday_auction_1(country)
            }
            MarketType::WholeSale(WholesaleMarketType::IntradayAuction2) => {
                Self::intraday_auction_2(country)
            }
            MarketType::WholeSale(WholesaleMarketType::IntradayAuction3) => {
                Self::intraday_auction_3(country)
            }
            MarketType::WholeSale(WholesaleMarketType::IntradayContinuous) => {
                Self::intraday_continuous(country)
            }
            MarketType::Ancillary(AncillaryMarketType::Energy(
                EnergyAncillaryMarketType::AfrrFree,
            )) => Self::afrr_free(country),
            MarketType::Ancillary(AncillaryMarketType::Capacity(
                CapacityAncillaryMarketType::Afrr,
            )) => Self::afrr(country),
            MarketType::Ancillary(AncillaryMarketType::Capacity(
                CapacityAncillaryMarketType::Fcr,
            )) => Self::fcr(country),
        }
    }

    fn spot_day_ahead(country: &Countries) -> crate::Result<Self> {
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
        Ok(Self::SpotDayAhead(MarketConfig { country: *country, auction }))
    }

    fn intraday_auction_1(country: &Countries) -> crate::Result<Self> {
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
        Ok(Self::IntradayAuction1(MarketConfig { country: *country, auction }))
    }
    fn intraday_auction_2(country: &Countries) -> crate::Result<Self> {
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
        Ok(Self::IntradayAuction2(MarketConfig { country: *country, auction }))
    }
    fn intraday_auction_3(country: &Countries) -> crate::Result<Self> {
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
        Ok(Self::IntradayAuction3(MarketConfig { country: *country, auction }))
    }

    fn intraday_continuous(country: &Countries) -> crate::Result<Self> {
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
        Ok(Self::IntradayContinuous(MarketConfig { country: *country, auction }))
    }

    fn afrr_free(country: &Countries) -> crate::Result<Self> {
        let harmonized_picasso = DynamicAuctionTemporality::try_new(
            TimeDefinedInterval::try_new(12, 0, 23, 59, 0.days())?,
            30.minutes(),
            "CET",
        )?;
        let auction = match country {
            Countries::BE
            | Countries::DE
            | Countries::IT
            | Countries::FR
            | Countries::ES
            | Countries::PT => harmonized_picasso,
        };
        Ok(Self::AfrrFree(MarketConfig { country: *country, auction }))
    }

    fn afrr(country: &Countries) -> crate::Result<Self> {
        Err(MarketError::NotImplemented {
            market: MarketType::Ancillary(AncillaryMarketType::Capacity(
                CapacityAncillaryMarketType::Afrr,
            )),
            country: *country,
        }
        .into())
    }

    fn fcr(country: &Countries) -> crate::Result<Self> {
        Err(MarketError::NotImplemented {
            market: MarketType::Ancillary(AncillaryMarketType::Capacity(
                CapacityAncillaryMarketType::Fcr,
            )),
            country: *country,
        }
        .into())
    }
}

// endregion: Market config
