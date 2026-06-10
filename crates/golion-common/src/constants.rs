// region: Add serde support macro.
macro_rules! serde_type {
    ($item:item) => {
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[derive(Debug, Hash, Clone, PartialEq, Eq)]
        $item
    };
}
// endregion: Add serde support macro.
// region: Market types
serde_type! {
    pub enum WholesaleMarketType {
        SpotDayAhead,
        IntradayAuction,
        IntradayContinuous,
    }
}

serde_type! {
    pub enum CertifiedAncillaryMarketType {
        Afrr,
        Fcr,
    }
}

serde_type! {
    pub enum UncertifiedAncillaryMarketType {
        AfrrFree,
    }
}

serde_type! {
    pub enum AncillaryMarketType {
        Certified(CertifiedAncillaryMarketType),
        Uncertified(UncertifiedAncillaryMarketType),
    }
}

serde_type! {
    pub enum MarketType {
        WholeSale(WholesaleMarketType),
        Ancillary(AncillaryMarketType),
    }
}
// endregion: Market types
// region: Countries
serde_type! {
    pub enum Countries {
        FR,
        DE,
        BE,
        ES,
        IT,
        PT,
    }
}
// endregion: Countries
