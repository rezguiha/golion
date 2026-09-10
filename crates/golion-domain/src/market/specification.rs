use crate::temporal::grid::RegularTimeGrid;
use crate::{
    countries::Countries,
    market::{
        bid::ProductSpecifications, config::AllMarketConfig, error::MarketError,
        market_type::MarketType, temporality::bid_time_bounds::ToBidTimeBounds,
    },
};
use jiff::Timestamp;
#[derive(Debug)]
pub struct MarketSpecs {
    pub market: MarketType,
    pub country: Countries,
    pub product: ProductSpecifications,
    pub config: AllMarketConfig,
}

impl MarketSpecs {
    pub fn try_new(
        market: MarketType,
        country: Countries,
        product: ProductSpecifications,
    ) -> crate::Result<Self> {
        let config = AllMarketConfig::try_new(&market, &country)?;
        if config.possible_products().contains(&product) {
            Ok(MarketSpecs { market, country, product, config })
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

impl MarketSpecs {
    /// Get bidding time bounds using the configuration and reference run time.
    pub fn get_bid_time_bounds(
        &self,
        reference_time: &Timestamp,
    ) -> crate::Result<Option<RegularTimeGrid>> {
        self.config.to_bid_time_bounds(reference_time, &self.product)
    }
}
