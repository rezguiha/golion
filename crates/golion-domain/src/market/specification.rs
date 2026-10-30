use jiff::{Timestamp, ToSpan};

use crate::countries::Countries;
use crate::market::bid::ProductSpecifications;
use crate::market::error::MarketError;
use crate::market::market_type::{
    AncillaryMarketType, CapacityAncillaryMarketType, EnergyAncillaryMarketType,
    MarketType, WholesaleMarketType,
};
use crate::market::product::{AncillaryProductDef, ProductDef, WholesaleProductDef};
use crate::market::temporality::auction::AuctionTemporality;
use crate::market::temporality::bid_window::BidWindow;
use crate::market::temporality::{
    auction::{
        ContinuousAuctionTemporality, DynamicAuctionTemporality, StaticAuctionTemporality,
    },
    interval::TimeDefinedInterval,
};
use crate::temporal::grid::RegularTimeGrid;
use crate::temporal::step::Step;

const WHOLESALE_INCREMENT_KW: u16 = 100;
const ANCILLARY_INCREMENT_KW: u16 = 1000;

// region: Market specs

/// Specification of one market in one country, with the product chosen to bid on.
///
/// The market functions below are the place to set any country specific
/// configuration: open and closure times, a neutralization delay translating a
/// liquidity constraint... They match every country explicitly so that adding a
/// country forces a decision for each market.
///
/// aFRR and FCR are not implemented yet and some configurations may be corrected
/// later on with further verifications.
#[derive(Debug)]
pub struct MarketSpecs {
    pub market: MarketType,
    /// Country of market.
    pub country: Countries,
    /// Auction temporality
    pub auction: AuctionTemporality,
    /// Chosen product (step and bid increment).
    pub product: ProductSpecifications,
}

impl MarketSpecs {
    pub fn get_bid_windows(
        &self,
        reference_time: Timestamp,
        grid: &RegularTimeGrid,
    ) -> crate::Result<Vec<BidWindow>> {
        self.auction.compute_bid_windows(&reference_time, grid, &self.product)
    }
}

// endregion: Market specs

// region: Wholesale Markets

/// Selects the requested product if its step is allowed on the market.
fn select_wholesale_product(
    product_def: WholesaleProductDef,
    allowed_steps: &[Step],
) -> crate::Result<ProductSpecifications> {
    if allowed_steps.contains(&product_def.step) {
        Ok(ProductSpecifications {
            step: product_def.step,
            increment: WHOLESALE_INCREMENT_KW.into(),
        })
    } else {
        Err(MarketError::InvalidWholesaleProduct {
            market: product_def.market,
            country: product_def.country,
            product_step: product_def.step,
            possible_product_steps: allowed_steps.to_vec(),
        }
        .into())
    }
}

fn spot_day_ahead(product_def: WholesaleProductDef) -> crate::Result<MarketSpecs> {
    let product =
        select_wholesale_product(product_def, &[Step::FifteenMinutes, Step::OneHour])?;
    let common_european_temporality = StaticAuctionTemporality::try_new(
        TimeDefinedInterval::try_new(0, 0, 12, 0, 0.days())?,
        TimeDefinedInterval::try_new(0, 0, 0, 0, 1.days())?,
        1.days(),
        "CET",
    )?;
    let auction = match product_def.country {
        Countries::BE
        | Countries::DE
        | Countries::IT
        | Countries::FR
        | Countries::ES
        | Countries::PT => common_european_temporality,
    };
    Ok(MarketSpecs {
        market: product_def.market.into(),
        country: product_def.country,
        auction: auction.into(),
        product,
    })
}

fn intraday_auction_1(product_def: WholesaleProductDef) -> crate::Result<MarketSpecs> {
    let product =
        select_wholesale_product(product_def, &[Step::FifteenMinutes, Step::OneHour])?;
    let harmonized_sidc_ida1 = StaticAuctionTemporality::try_new(
        TimeDefinedInterval::try_new(0, 0, 15, 0, 0.days())?,
        TimeDefinedInterval::try_new(0, 0, 0, 0, 1.days())?,
        1.days(),
        "CET",
    )?;
    let auction = match product_def.country {
        Countries::BE
        | Countries::DE
        | Countries::IT
        | Countries::FR
        | Countries::ES
        | Countries::PT => harmonized_sidc_ida1,
    };
    Ok(MarketSpecs {
        market: product_def.market.into(),
        country: product_def.country,
        auction: auction.into(),
        product,
    })
}
fn intraday_auction_2(product_def: WholesaleProductDef) -> crate::Result<MarketSpecs> {
    let product =
        select_wholesale_product(product_def, &[Step::FifteenMinutes, Step::OneHour])?;
    let harmonized_sidc_ida2 = StaticAuctionTemporality::try_new(
        TimeDefinedInterval::try_new(0, 0, 22, 0, 0.days())?,
        TimeDefinedInterval::try_new(0, 0, 0, 0, 1.days())?,
        1.days(),
        "CET",
    )?;
    let auction = match product_def.country {
        Countries::BE
        | Countries::DE
        | Countries::IT
        | Countries::FR
        | Countries::ES
        | Countries::PT => harmonized_sidc_ida2,
    };
    Ok(MarketSpecs {
        market: product_def.market.into(),
        country: product_def.country,
        auction: auction.into(),
        product,
    })
}
fn intraday_auction_3(product_def: WholesaleProductDef) -> crate::Result<MarketSpecs> {
    let product =
        select_wholesale_product(product_def, &[Step::FifteenMinutes, Step::OneHour])?;
    let harmonized_sidc_ida3 = StaticAuctionTemporality::try_new(
        TimeDefinedInterval::try_new(0, 0, 10, 0, 0.days())?,
        TimeDefinedInterval::try_new(12, 0, 0, 0, 1.days())?,
        0.days(),
        "CET",
    )?;
    let auction = match product_def.country {
        Countries::BE
        | Countries::DE
        | Countries::IT
        | Countries::FR
        | Countries::ES
        | Countries::PT => harmonized_sidc_ida3,
    };
    Ok(MarketSpecs {
        market: product_def.market.into(),
        country: product_def.country,
        auction: auction.into(),
        product,
    })
}

fn intraday_continuous(product_def: WholesaleProductDef) -> crate::Result<MarketSpecs> {
    let product = select_wholesale_product(
        product_def,
        &[Step::FifteenMinutes, Step::ThirtyMinutes, Step::OneHour],
    )?;
    let auction = match product_def.country {
        Countries::FR | Countries::ES | Countries::PT | Countries::IT => {
            ContinuousAuctionTemporality::try_new(15, 0, 1.hours(), "CET")?
        }
        Countries::DE => {
            ContinuousAuctionTemporality::try_new(15, 0, 15.minutes(), "CET")?
        }
        Countries::BE => ContinuousAuctionTemporality::try_new(14, 0, 1.hours(), "CET")?,
    };
    Ok(MarketSpecs {
        market: product_def.market.into(),
        country: product_def.country,
        auction: auction.into(),
        product,
    })
}

// endregion: Wholesale Markets

// region: Ancillary Services
fn afrr_free(product_def: AncillaryProductDef) -> crate::Result<MarketSpecs> {
    let harmonized_picasso = DynamicAuctionTemporality::try_new(
        TimeDefinedInterval::try_new(0, 0, 0, 0, 1.days())?,
        30.minutes(),
        "CET",
    )?;
    let auction = match product_def.country {
        Countries::BE
        | Countries::DE
        | Countries::IT
        | Countries::FR
        | Countries::ES
        | Countries::PT => harmonized_picasso,
    };
    let product_specs: ProductSpecifications =
        ProductSpecifications::try_new(15.minutes(), ANCILLARY_INCREMENT_KW)?;
    Ok(MarketSpecs {
        market: product_def.market.into(),
        country: product_def.country,
        auction: auction.into(),
        product: product_specs,
    })
}

fn afrr(product_def: AncillaryProductDef) -> crate::Result<MarketSpecs> {
    Err(MarketError::NotImplemented {
        market: MarketType::Ancillary(AncillaryMarketType::Capacity(
            CapacityAncillaryMarketType::Afrr,
        )),
        country: product_def.country,
    }
    .into())
}

fn fcr(product_def: AncillaryProductDef) -> crate::Result<MarketSpecs> {
    let harmonized = StaticAuctionTemporality::try_new(
        TimeDefinedInterval::try_new(0, 0, 8, 0, 0.days())?,
        TimeDefinedInterval::try_new(0, 0, 0, 0, 1.days())?,
        1.days(),
        "CET",
    )?;
    let auction: crate::Result<StaticAuctionTemporality> = match product_def.country {
        Countries::BE | Countries::DE | Countries::FR => Ok(harmonized),
        Countries::ES | Countries::PT | Countries::IT => {
            Err(MarketError::NotImplemented {
                market: MarketType::Ancillary(AncillaryMarketType::Capacity(
                    CapacityAncillaryMarketType::Fcr,
                )),
                country: product_def.country,
            }
            .into())
        }
    };

    let product_specs: ProductSpecifications =
        ProductSpecifications::try_new(4.hours(), ANCILLARY_INCREMENT_KW)?;
    Ok(MarketSpecs {
        market: product_def.market.into(),
        country: product_def.country,
        auction: auction?.into(),
        product: product_specs,
    })
}
// endregion: Ancillary Services

// region: Conversions
impl TryFrom<AncillaryProductDef> for MarketSpecs {
    type Error = crate::Error;
    fn try_from(value: AncillaryProductDef) -> crate::Result<Self> {
        match value.market {
            AncillaryMarketType::Energy(EnergyAncillaryMarketType::AfrrFree) => {
                afrr_free(value)
            }
            AncillaryMarketType::Capacity(CapacityAncillaryMarketType::Afrr) => {
                afrr(value)
            }
            AncillaryMarketType::Capacity(CapacityAncillaryMarketType::Fcr) => fcr(value),
        }
    }
}

impl TryFrom<WholesaleProductDef> for MarketSpecs {
    type Error = crate::Error;
    fn try_from(value: WholesaleProductDef) -> crate::Result<Self> {
        match value.market {
            WholesaleMarketType::SpotDayAhead => spot_day_ahead(value),
            WholesaleMarketType::IntradayAuction1 => intraday_auction_1(value),
            WholesaleMarketType::IntradayAuction2 => intraday_auction_2(value),
            WholesaleMarketType::IntradayAuction3 => intraday_auction_3(value),
            WholesaleMarketType::IntradayContinuous => intraday_continuous(value),
        }
    }
}

impl From<&MarketSpecs> for ProductDef {
    fn from(value: &MarketSpecs) -> Self {
        match value.market {
            MarketType::Ancillary(a) => {
                AncillaryProductDef { country: value.country, market: a }.into()
            }
            MarketType::WholeSale(w) => WholesaleProductDef {
                country: value.country,
                market: w,
                step: value.product.step,
            }
            .into(),
        }
    }
}
// endregion: Conversions
