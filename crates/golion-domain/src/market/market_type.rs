use crate::serde_type;
// region: Market types
serde_type! {
    #[derive(Debug, Hash, Clone, PartialEq, Eq)]
    pub enum WholesaleMarketType {
        SpotDayAhead,
        IntradayAuction,
        IntradayContinuous,
    }
}

serde_type! {
    #[derive(Debug, Hash, Clone, PartialEq, Eq)]
    pub enum CertifiedAncillaryMarketType {
        Afrr,
        Fcr,
    }
}

serde_type! {
    #[derive(Debug, Hash, Clone, PartialEq, Eq)]
    pub enum UncertifiedAncillaryMarketType {
        AfrrFree,
    }
}

serde_type! {
    #[derive(Debug, Hash, Clone, PartialEq, Eq)]
    pub enum AncillaryMarketType {
        Certified(CertifiedAncillaryMarketType),
        Uncertified(UncertifiedAncillaryMarketType),
    }
}

serde_type! {
    #[derive(Debug, Hash, Clone, PartialEq, Eq)]
    pub enum MarketType {
        WholeSale(WholesaleMarketType),
        Ancillary(AncillaryMarketType),
    }
}
// endregion: Market types
