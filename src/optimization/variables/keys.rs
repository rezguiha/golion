use crate::constants::{Countries, MarketType};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Hash key to access physical variables
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicalKey {
    asset_id: Uuid,
    time: DateTime<Utc>,
}

/// Hash key to access bidding variables
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BiddingKey {
    asset_id: Uuid,
    market_type: MarketType,
    country: Countries,
    time: DateTime<Utc>,
}
