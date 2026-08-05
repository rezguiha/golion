use super::certification::CertifiedEnvelope;
use golion_domain::countries::Countries;
/// Definition of market choices for assets and their inputs.
use golion_domain::market::market_type::{
    CapacityAncillaryMarketType, EnergyAncillaryMarketType, WholesaleMarketType,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
pub enum MarketChoice {
    WholeSaleChoice {
        market: WholesaleMarketType,
        country: Countries,
    },
    CapacityAncillaryChoice {
        market: CapacityAncillaryMarketType,
        country: Countries,
        certified: CertifiedEnvelope,
    },
    EnergyAncillaryChoice {
        market: EnergyAncillaryMarketType,
        country: Countries,
    },
}
