/// Crate Constant definitions
// region: Market types
/// Wholesale markets enumeration.
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum WholesaleMarketType {
    SpotDayAhead,
    IntradayAuction,
    IntradayContinuous,
}
/// Ancillary Services enumeration.
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum AncillaryMarketType {
    AfrrFree,
    Afrr,
    Fcr,
}
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
/// All market enumeration.
pub enum MarketType {
    WholeSale(WholesaleMarketType),
    Ancillary(AncillaryMarketType),
}
// endregion: Market types

// region: Countries
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum Countries {
    FR,
    DE,
    BE,
    ES,
    IT,
    PT,
}
// endregions: Countries
