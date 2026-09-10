use crate::temporal::grid::RegularTimeGrid;
use crate::{
    countries::Countries,
    market::{
        bid::BidSpecs, config::AllMarketConfig, error::MarketError,
        market_type::MarketType, temporality::bid_time_bounds::ToBidTimeBounds,
    },
};
use jiff::Timestamp;
#[derive(Debug)]
pub struct MarketSpec {
    pub market: MarketType,
    pub country: Countries,
    pub product: BidSpecs,
    pub config: AllMarketConfig,
}

impl MarketSpec {
    pub fn try_new(
        market: MarketType,
        country: Countries,
        product: BidSpecs,
    ) -> crate::Result<Self> {
        let config = AllMarketConfig::try_new(&market, &country)?;
        if config.possible_products().contains(&product) {
            Ok(MarketSpec { market, country, product, config })
        } else {
            Err(MarketError::InvalidProduct {
                market,
                country,
                product,
                possible_products: config.possible_products().clone(),
            }
            .into())
        }
    }
}

impl MarketSpec {
    pub fn get_bid_time_bounds(
        &self,
        reference_time: &Timestamp,
    ) -> crate::Result<Option<RegularTimeGrid>> {
        self.config.to_bid_time_bounds(reference_time, &self.product)
    }
}
