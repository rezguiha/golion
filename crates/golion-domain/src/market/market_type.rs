use serde::{Deserialize, Serialize};
// region: Market types

#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WholesaleMarketType {
    SpotDayAhead,
    IntradayAuction1,
    IntradayAuction2,
    IntradayAuction3,
    IntradayContinuous,
}
/// A capacity ancillary market is a market where we sell "capacity"
/// which represents a commitment to be available to deliver an agreed
/// upon amount of power either to charge or discharge. This is decided
/// D-1 and commitments are in D. You are paid for your availability
/// and you can be activated or not on your commitment in D on which
/// you are paid an extra variable amount depending on the level of power
/// you and the time you spend activated. This is called the "energy"
/// component of an ancillary market.
#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapacityAncillaryMarketType {
    Afrr,
    Fcr,
}
/// An energy ancillary market is a market where you opt in to participate
/// to the energy component of the ancillary capacity market.
#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnergyAncillaryMarketType {
    AfrrFree,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AncillaryMarketType {
    Capacity(CapacityAncillaryMarketType),
    Energy(EnergyAncillaryMarketType),
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MarketType {
    WholeSale(WholesaleMarketType),
    Ancillary(AncillaryMarketType),
}

impl From<WholesaleMarketType> for MarketType {
    fn from(value: WholesaleMarketType) -> Self {
        Self::WholeSale(value)
    }
}

impl From<CapacityAncillaryMarketType> for MarketType {
    fn from(value: CapacityAncillaryMarketType) -> Self {
        Self::Ancillary(AncillaryMarketType::Capacity(value))
    }
}

impl From<EnergyAncillaryMarketType> for MarketType {
    fn from(value: EnergyAncillaryMarketType) -> Self {
        Self::Ancillary(AncillaryMarketType::Energy(value))
    }
}

// endregion: Market types
