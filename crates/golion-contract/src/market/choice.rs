use golion_domain::market::bid::ProductSpecifications;
/// Definition of market choices for assets and their inputs.
use golion_domain::market::specification::MarketSpecs;
use golion_domain::{countries::Countries, market::market_type::MarketType};
use jiff::Span;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MarketChoice {
    pub market: MarketType,
    pub country: Countries,
    /// Wholesale product length expressed in minutes.
    pub product_step_minutes: u16,
    pub product_increment_kw: u16,
}

impl TryFrom<MarketChoice> for MarketSpecs {
    type Error = crate::Error;
    fn try_from(value: MarketChoice) -> Result<Self, Self::Error> {
        let product = ProductSpecifications::try_new(
            Span::new().minutes(value.product_step_minutes),
            value.product_increment_kw,
        )?;
        Ok(MarketSpecs::try_new(value.market, value.country, product)?)
    }
}
