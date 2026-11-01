/// Definition of market choices for assets and their inputs.
use lib_domain::market::market_type::{AncillaryMarketType, WholesaleMarketType};
use lib_domain::market::product::{AncillaryProductDef, WholesaleProductDef};

use jiff::SignedDuration;
use lib_domain::countries::Countries;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct AncillaryChoice {
    pub market: AncillaryMarketType,
}
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct WholesaleChoice {
    pub market: WholesaleMarketType,
    pub product_step: SignedDuration,
}
impl AncillaryChoice {
    pub(crate) fn to_product_def(self, country: Countries) -> AncillaryProductDef {
        AncillaryProductDef { country, market: self.market }
    }
}

impl WholesaleChoice {
    pub(crate) fn try_to_product_def(
        self,
        country: Countries,
    ) -> crate::Result<WholesaleProductDef> {
        Ok(WholesaleProductDef {
            market: self.market,
            step: self.product_step.try_into()?,
            country,
        })
    }
}
