use crate::market::choice::MarketChoice;
use garde::Validate;
use golion_domain::countries::Countries;
use golion_domain::market::commitment::PowerCommitment;
use golion_domain::market::market_type::AncillaryMarketType;
use golion_domain::problem::definition::ReserveDefinition;
use golion_domain::temporal::grid::RegularTimeGrid;
use jiff::SignedDuration;
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
    // Worst case full activation projection window
    #[garde(skip)]
    pub activation_window: SignedDuration,
}

// region: Domain Conversion
impl ReservePerimeter {
    /// Converts the perimeter payload into its domain definition.
    pub fn try_into_definition(
        self,
        country: &Countries,
        grid: &RegularTimeGrid,
    ) -> crate::Result<ReserveDefinition> {
        Ok(ReserveDefinition::try_new(
            self.id,
            self.market.try_into_market_specs(country)?,
            self.composition,
            self.commitments.iter().map(PowerCommitment::from),
            grid,
            self.activation_window.try_into()?,
        )?)
    }
}
// endregion: Domain Conversion
