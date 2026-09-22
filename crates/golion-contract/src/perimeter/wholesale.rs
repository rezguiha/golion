use golion_domain::countries::Countries;
use golion_domain::market::market_type::WholesaleMarketType;
use golion_domain::problem::definition::BrpDefinition;
use golion_domain::temporal::step::MinuteStep;
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

// region: Domain Conversion
impl BrpPerimeter {
    /// Converts the perimeter payload into its domain definition, using
    /// the optimization step to turn net positions into powers.
    pub fn try_into_definition(
        self,
        country: &Countries,
        step: &MinuteStep,
    ) -> crate::Result<BrpDefinition> {
        let markets = self
            .markets
            .iter()
            .map(|market_choice| market_choice.try_into_market_specs(country))
            .collect::<crate::Result<_>>()?;
        let commitments = self
            .commitments
            .iter()
            .flat_map(|series| {
                series.values.iter().map(|commitment| commitment.to_commitment(step))
            })
            .collect();
        Ok(BrpDefinition {
            id: self.id,
            composition: self.composition,
            markets,
            commitments,
        })
    }
}
// endregion: Domain Conversion
