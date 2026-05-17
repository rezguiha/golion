use super::certification::CertifiedEnvelope;
/// Definition of market choices for assets and their inputs.
use crate::constants::{
    CertifiedAncillaryMarketType, Countries, UncertifiedAncillaryMarketType,
    WholesaleMarketType,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
pub enum MarketChoice {
    WholeSaleChoice {
        market: WholesaleMarketType,
        country: Countries,
    },
    CertifiedAncillaryChoice {
        market: CertifiedAncillaryMarketType,
        country: Countries,
        certified: CertifiedEnvelope,
    },
    UncertifiedAncillaryChoice {
        market: UncertifiedAncillaryMarketType,
        country: Countries,
    },
}
