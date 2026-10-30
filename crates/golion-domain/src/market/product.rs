use derive_more::{Eq, From};

use crate::{
    countries::Countries,
    market::market_type::{AncillaryMarketType, WholesaleMarketType},
    temporal::step::Step,
};
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct WholesaleProductDef {
    pub country: Countries,
    pub market: WholesaleMarketType,
    pub step: Step,
}
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct AncillaryProductDef {
    pub country: Countries,
    pub market: AncillaryMarketType,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, From)]
pub enum ProductDef {
    Wholesale(WholesaleProductDef),
    Ancillary(AncillaryProductDef),
}
