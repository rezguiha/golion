use golion_domain::market::market_type::WholesaleMarketType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::market::{choice::MarketChoice, commitments::WholesaleCommitments};
use garde::Validate;
/// Represents the balance responsible party identification.
/// It is the entity facing the TSOs and on which imbalances
/// are measured and paid.
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct BrpPerimeter {
    #[garde(skip)]
    pub id: Uuid,
    /// List of asset ids composing the perimiter on which the balancing
    /// responsible party operates.
    #[garde(skip)]
    pub composition: Vec<Uuid>,
    /// List of wholesale markets to bid on.
    #[garde(skip)]
    pub markets: Vec<MarketChoice<WholesaleMarketType>>,
    /// List of wholesale market commitments at the perimeter.
    #[garde(dive)]
    pub commitments: WholesaleCommitments,
}
