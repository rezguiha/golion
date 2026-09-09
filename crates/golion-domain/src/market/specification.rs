use crate::{
    countries::Countries,
    market::{
        bid::BidSpecs, config::AllMarketConfig, error::MarketError,
        market_type::MarketType,
    },
};

#[derive(Debug)]
pub struct MarketSpec {
    market: MarketType,
    country: Countries,
    product: BidSpecs,
}

impl MarketSpec {
    pub fn try_new(
        market: MarketType,
        country: Countries,
        product: BidSpecs,
    ) -> crate::Result<Self> {
        let config = AllMarketConfig::try_new(&market, &country)?;
        if config.possible_products().contains(&product) {
            Ok(MarketSpec { market, country, product })
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
