use golion_domain::market::market_type::AncillaryMarketType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use garde::Validate;
use typed_builder::TypedBuilder;

use crate::market::commitments::AncillaryCommitment;
/// Represents the reserve perimeter
#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder)]
pub struct ReservePerimeter {
    #[garde(skip)]
    pub id: Uuid,
    #[garde(skip)]
    pub market: AncillaryMarketType,
    /// List of reserve units/groups composing the reserve perimeter
    /// for that particular ancillary service.
    #[garde(skip)]
    pub composition: Vec<Uuid>,
    /// List of commitments of the perimeter for that particular
    /// ancillary service.
    #[garde(dive)]
    pub commitments: Vec<AncillaryCommitment>,
}
