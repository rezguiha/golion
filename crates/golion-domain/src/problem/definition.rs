/// Definitions of the elements taking part in an optimization problem.
/// They describe what to optimize, independently of how it is modelled.
use crate::asset::bess::specification::BessSpecifications;
use crate::market::commitment::Commitment;
use crate::market::specification::MarketSpecs;
use crate::units::power::KiloWattHour;
use uuid::Uuid;

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
    pub id: Uuid,
    /// Ids of the assets composing the perimeter.
    pub composition: Vec<Uuid>,
    /// Wholesale markets the perimeter bids on.
    pub markets: Vec<MarketSpecs>,
    /// Commitments already taken on all wholesale markets.
    pub commitments: Vec<Commitment>,
}

/// Reserve perimeter: assets certified together for one ancillary service.
#[derive(Debug)]
pub struct ReserveDefinition {
    pub id: Uuid,
    /// Ancillary service the perimeter bids on.
    pub market: MarketSpecs,
    /// Ids of the assets composing the perimeter.
    pub composition: Vec<Uuid>,
    /// Commitments already taken on the ancillary service.
    pub commitments: Vec<Commitment>,
}
// endregion: Perimeter Definitions
