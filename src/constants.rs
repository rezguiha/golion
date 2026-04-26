/// Crate Constant definitions

// region: Market types
/// Wholesale markets enumeration.
pub enum WholesaleMarketType {
    SpotDayAhead,
    IntradayAuction,
    IntradayContinuous,
}
/// Ancillary Services enumeration.
pub enum AncillaryMarketType {
    AfrrFree,
    Afrr,
    Fcr,
}
/// All market enumeration.
pub enum MarketType {
    WholeSale(WholesaleMarketType),
    Ancillary(AncillaryMarketType),
}
// endregion: Market types
