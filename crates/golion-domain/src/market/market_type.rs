use serde::{Deserialize, Serialize};
// region: Market types

#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WholesaleMarketType {
    SpotDayAhead,
    IntradayAuction,
    IntradayContinuous,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertifiedAncillaryMarketType {
    Afrr,
    Fcr,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UncertifiedAncillaryMarketType {
    AfrrFree,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AncillaryMarketType {
    Certified(CertifiedAncillaryMarketType),
    Uncertified(UncertifiedAncillaryMarketType),
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MarketType {
    WholeSale(WholesaleMarketType),
    Ancillary(AncillaryMarketType),
}

// endregion: Market types
