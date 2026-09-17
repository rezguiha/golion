/// Definition of market choices for assets and their inputs.
use golion_domain::market::bid::ProductSpecifications;
use golion_domain::market::specification::MarketSpecs;
use golion_domain::{countries::Countries, market::market_type::MarketType};
use jiff::Span;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MarketChoice<T: Into<MarketType>> {
    pub market: T,
    pub product_step_minutes: u16,
    pub product_increment_kw: u16,
}

impl<T: Into<MarketType> + Copy> MarketChoice<T> {
    pub fn try_into_market_specs(
        &self,
        country: &Countries,
    ) -> crate::Result<MarketSpecs> {
        let product = ProductSpecifications::try_new(
            Span::new().minutes(self.product_step_minutes),
            self.product_increment_kw,
        )?;
        Ok(MarketSpecs::try_new(self.market.into(), *country, product)?)
    }
}
