use crate::{
    countries::Countries,
    market::market_type::{AncillaryMarketType, WholesaleMarketType},
    temporal::step::Step,
};
#[derive(Debug, Clone, Copy, Hash)]
pub struct WholesaleProductDef {
    pub country: Countries,
    pub market: WholesaleMarketType,
    pub step: Step,
}
#[derive(Debug, Clone, Copy, Hash)]
pub struct AncillaryProductDef {
    pub country: Countries,
    pub market: AncillaryMarketType,
}
