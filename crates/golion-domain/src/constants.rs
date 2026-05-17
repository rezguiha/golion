/// Crate Constant definitions
// region: Market types
/// Wholesale markets enumeration.
use serde::Deserialize;
#[derive(Debug, Hash, Clone, PartialEq, Eq, Deserialize)]
pub enum WholesaleMarketType {
    SpotDayAhead,
    IntradayAuction,
    IntradayContinuous,
}
/// Ancillary Services enumeration.
#[derive(Debug, Hash, Clone, PartialEq, Eq, Deserialize)]
pub enum CertifiedAncillaryMarketType {
    Afrr,
    Fcr,
}
#[derive(Debug, Hash, Clone, PartialEq, Eq, Deserialize)]
pub enum UncertifiedAncillaryMarketType {
    AfrrFree,
}
#[derive(Debug, Hash, Clone, PartialEq, Eq, Deserialize)]
pub enum AncillaryMarketType {
    CertifiedAncillaryMarketType,
    UncertifiedAncillaryMarketType,
}
#[derive(Debug, Hash, Clone, PartialEq, Eq, Deserialize)]
/// All market enumeration.
pub enum MarketType {
    WholeSale(WholesaleMarketType),
    Ancillary(AncillaryMarketType),
}
// endregion: Market types

// region: Countries
#[derive(Debug, Hash, Clone, PartialEq, Eq, Deserialize)]
pub enum Countries {
    FR,
    DE,
    BE,
    ES,
    IT,
    PT,
}
// endregions: Countries
