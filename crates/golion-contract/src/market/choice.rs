/// Definition of market choices for assets and their inputs.
use golion_domain::market::market_type::{AncillaryMarketType, WholesaleMarketType};
use golion_domain::market::product::{AncillaryProductDef, WholesaleProductDef};

use golion_domain::countries::Countries;
use jiff::SignedDuration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AncillaryChoice {
    pub market: AncillaryMarketType,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct WholesaleChoice {
    pub market: WholesaleMarketType,
    pub product_step: SignedDuration,
}
impl AncillaryChoice {
    pub(crate) fn to_product_def(&self, country: Countries) -> AncillaryProductDef {
        AncillaryProductDef { country, market: self.market }
    }
}

impl WholesaleChoice {
    pub(crate) fn try_to_product_def(
        &self,
        country: Countries,
    ) -> crate::Result<WholesaleProductDef> {
        Ok(WholesaleProductDef {
            market: self.market,
            step: self.product_step.try_into()?,
            country,
        })
    }
}
