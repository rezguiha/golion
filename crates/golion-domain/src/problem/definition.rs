/// Definitions of the elements taking part in an optimization problem.
/// They describe what to optimize, independently of how it is modelled.
use crate::asset::bess::specification::BessSpecifications;
use crate::market::commitment::Commitment;
use crate::market::specification::MarketSpecs;
use crate::units::power::KiloWattHour;
use uuid::Uuid;
// Fixed penalty for now set here. May change if having it as an input of
// optimization may be relevant.
const ANCILLARY_PENALTY_EURO_PER_KW: f64 = -10.0;
const WHOLESALE_PENALTY_EURO_PER_KW: f64 = -4.0;

// region: Asset Definition
/// Asset physical specifications along with its state at the start
/// of the optimization run.
#[derive(Debug)]
pub enum AssetDefinition {
    Bess { specifications: BessSpecifications, initial_soc: KiloWattHour },
}
// endregion: Asset Definition

// region: Perimeter Definitions
/// Balance responsible party perimeter: assets whose net position is
/// traded together on wholesale markets and on which you pay imbalances.
#[derive(Debug)]
pub struct BrpDefinition {
    id: Uuid,
    /// Ids of the assets composing the perimeter.
    composition: Vec<Uuid>,
    /// Wholesale markets the perimeter bids on.
    markets: Vec<MarketSpecs>,
    /// Commitments already taken on all wholesale markets.
    commitments: Vec<Commitment>,
    /// Penalty for violations in euro per kW.
    penalty: f64,
}
impl BrpDefinition {
    pub fn new(
        id: Uuid,
        markets: Vec<MarketSpecs>,
        composition: Vec<Uuid>,
        commitments: Vec<Commitment>,
    ) -> Self {
        Self {
            id,
            markets,
            composition,
            commitments,
            penalty: ANCILLARY_PENALTY_EURO_PER_KW,
        }
    }
    pub fn id(&self) -> &Uuid {
        &self.id
    }
    pub fn markets(&self) -> &[MarketSpecs] {
        &self.markets
    }
    pub fn commitments(&self) -> &[Commitment] {
        &self.commitments
    }
    pub fn composition(&self) -> &[Uuid] {
        &self.composition
    }
    pub fn penalty(&self) -> &f64 {
        &self.penalty
    }
}
/// Reserve perimeter: assets certified together for one ancillary service.
#[derive(Debug)]
pub struct ReserveDefinition {
    id: Uuid,
    /// Ancillary service the perimeter bids on.
    market: MarketSpecs,
    /// Ids of the assets composing the perimeter.
    composition: Vec<Uuid>,
    /// Commitments already taken on the ancillary service.
    commitments: Vec<Commitment>,
    /// Penalty for violations in euro per kW.
    penalty: f64,
}
impl ReserveDefinition {
    pub fn new(
        id: Uuid,
        market: MarketSpecs,
        composition: Vec<Uuid>,
        commitments: Vec<Commitment>,
    ) -> Self {
        Self {
            id,
            market,
            composition,
            commitments,
            penalty: ANCILLARY_PENALTY_EURO_PER_KW,
        }
    }
    pub fn id(&self) -> &Uuid {
        &self.id
    }
    pub fn market(&self) -> &MarketSpecs {
        &self.market
    }
    pub fn commitments(&self) -> &[Commitment] {
        &self.commitments
    }
    pub fn composition(&self) -> &[Uuid] {
        &self.composition
    }
    pub fn penalty(&self) -> &f64 {
        &self.penalty
    }
}
// endregion: Perimeter Definitions
