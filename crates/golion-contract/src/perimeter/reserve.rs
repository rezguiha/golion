use garde::Validate;
use golion_domain::market::commitment::PowerCommitment;
use golion_domain::problem::definition::ReserveDefinition;
use golion_domain::temporal::grid::RegularTimeGrid;
use golion_domain::{countries::Countries, market::activation::ExpectedActivationRatios};
use jiff::SignedDuration;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::market::choice::AncillaryChoice;
use crate::market::commitments::AncillaryCommitment;
/// Represents the reserve perimeter
#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder)]
pub struct ReservePerimeter {
    #[garde(skip)]
    pub id: Uuid,
    #[garde(skip)]
    pub market: AncillaryChoice,
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
    #[garde(range(min = 0.0, max = 1.0))]
    pub upward_expected_activation_ratio: f64,
    #[garde(range(min = 0.0, max = 1.0))]
    pub downward_expected_activation_ratio: f64,
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
            self.market.to_product_def(*country).try_into()?,
            self.composition,
            self.commitments.iter().map(PowerCommitment::from),
            grid,
            self.activation_window,
            ExpectedActivationRatios::try_new(
                self.upward_expected_activation_ratio,
                self.downward_expected_activation_ratio,
            )?,
        )?)
    }
}
// endregion: Domain Conversion
