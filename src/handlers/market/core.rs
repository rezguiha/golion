use uuid::Uuid;
use chrono::{DateTime,Utc};
use crate::constants::MarketType;


/// Hash key to access bidding variables
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BiddingKey{
    asset_id: Uuid,
    market_type:MarketType,
    time: DateTime<Utc>
    
}