use crate::market::choice::MarketChoice;
use garde::Validate;
use golion_domain::countries::Countries;
use golion_domain::market::commitment::Commitment;
use golion_domain::market::market_type::AncillaryMarketType;
use golion_domain::problem::definition::ReserveDefinition;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::market::commitments::AncillaryCommitment;
/// Represents the reserve perimeter
#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder)]
pub struct ReservePerimeter {
    #[garde(skip)]
    pub id: Uuid,
    #[garde(skip)]
    pub market: MarketChoice<AncillaryMarketType>,
    /// List of reserve units/groups composing the reserve perimeter
    /// for that particular ancillary service.
    #[garde(skip)]
    pub composition: Vec<Uuid>,
    /// List of commitments of the perimeter for that particular
    /// ancillary service.
    #[garde(dive)]
    pub commitments: Vec<AncillaryCommitment>,
}

// region: Domain Conversion
impl ReservePerimeter {
    /// Converts the perimeter payload into its domain definition.
    pub fn try_into_definition(
        self,
        country: &Countries,
    ) -> crate::Result<ReserveDefinition> {
        Ok(ReserveDefinition {
            id: self.id,
            market: self.market.try_into_market_specs(country)?,
            composition: self.composition,
            commitments: self.commitments.iter().map(Commitment::from).collect(),
        })
    }
}
// endregion: Domain Conversion
