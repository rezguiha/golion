use crate::countries::Countries;
use crate::market::error::MarketError;
use crate::market::market_type::MarketType;
use crate::temporal::series::{TimeSeries, TimeStampedUtc};
use derive_more::From;
use jiff::Timestamp;
use std::collections::HashMap;

// region: Revenue
/// Unit the revenue rate is expressed against, which decides whether it
/// applies to a power bid or to the energy delivered over the bid window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevenueUnit {
    /// €/kW of bid power.
    PerKiloWatt,
    /// €/kWh of bid energy.
    PerKiloWattHour,
}

#[derive(Debug)]
pub struct Revenue {
    pub start_at: Timestamp,
    pub unit: RevenueUnit,
    pub input_revenue: f64,
    pub output_revenue: f64,
}

impl TimeStampedUtc for Revenue {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}
// endregion: Revenue

// region: Revenue store
/// Revenue series of every market taking part in the optimization,
/// indexed by the market and the country it is run in.
#[derive(Debug, Default, From)]
pub struct RevenueStore(HashMap<MarketType, TimeSeries<Revenue>>);

impl RevenueStore {
    pub fn get(
        &self,
        market: MarketType,
        country: Countries,
    ) -> crate::Result<&TimeSeries<Revenue>> {
        self.0
            .get(&market)
            .ok_or_else(|| MarketError::MissingMarketRevenue { market, country }.into())
    }
}
// endregion: Revenue store
