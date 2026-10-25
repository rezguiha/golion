use crate::market::temporality::bid_window::BidWindow;
use crate::temporal::grid::RegularTimeGrid;
use crate::{
    countries::Countries,
    market::{
        bid::ProductSpecifications, config::AllMarketConfig, error::MarketError,
        market_type::MarketType,
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
    /// Get bidding time windows using the configuration and reference run time.
    pub fn get_bid_windows(
        &self,
        grid: &RegularTimeGrid,
        reference_time: &Timestamp,
    ) -> crate::Result<Vec<BidWindow>> {
        self.config.to_bid_windows(reference_time, grid, &self.product)
    }
}
