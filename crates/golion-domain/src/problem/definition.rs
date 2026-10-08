/// Definitions of the elements taking part in an optimization problem.
/// They describe what to optimize, independently of how it is modelled.
use crate::asset::bess::specification::BessSpecifications;
use crate::market::commitment::{EnergyCommitment, PowerCommitment};
use crate::market::specification::MarketSpecs;
use crate::temporal::grid::RegularTimeGrid;
use crate::temporal::series::TimeSeries;
use crate::temporal::step::MinuteStep;
use crate::units::power::{KiloWatt, KiloWattHour};
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
impl AssetDefinition {
    pub(crate) fn max_input_power(&self) -> KiloWatt {
        match self {
            Self::Bess { specifications, .. } => specifications.rated_charge_power,
        }
    }
    pub(crate) fn max_output_power(&self) -> KiloWatt {
        match self {
            Self::Bess { specifications, .. } => specifications.rated_discharge_power,
        }
    }
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
    commitments: TimeSeries<PowerCommitment>,
    /// Penalty for violations in euro per kW.
    penalty: f64,
}
impl BrpDefinition {
    pub fn try_new(
        id: Uuid,
        markets: Vec<MarketSpecs>,
        composition: Vec<Uuid>,
        commitments: impl Iterator<Item = EnergyCommitment>,
        grid: &RegularTimeGrid,
    ) -> crate::Result<Self> {
        // One net energy per grid slot, zero where nothing is committed.
        let mut net_positions: Vec<f64> = grid.iter().map(|_| 0.0).collect();
        for commitment in commitments {
            let index = grid.index_of(&commitment.start_at)?;
            net_positions[index] += commitment.energy_net_position.0;
        }
        // Split each slot into input/output power only once it is netted.
        let commitments: Vec<_> = grid
            .iter()
            .zip(net_positions)
            .map(|(start_at, net_position)| {
                EnergyCommitment { start_at, energy_net_position: net_position.into() }
                    .to_power_commitment(grid.step())
            })
            .collect();
        Ok(Self {
            id,
            markets,
            composition,
            commitments: commitments.try_into()?,
            penalty: WHOLESALE_PENALTY_EURO_PER_KW,
        })
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }
    pub fn markets(&self) -> &[MarketSpecs] {
        &self.markets
    }
    pub fn commitments(&self) -> &TimeSeries<PowerCommitment> {
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
    commitments: TimeSeries<PowerCommitment>,
    /// Penalty for violations in euro per kW.
    penalty: f64,
    // Full activation projection window.
    activation_window: MinuteStep,
}
impl ReserveDefinition {
    pub fn try_new(
        id: Uuid,
        market: MarketSpecs,
        composition: Vec<Uuid>,
        commitments: impl Iterator<Item = PowerCommitment>,
        grid: &RegularTimeGrid,
        activation_window: MinuteStep,
    ) -> crate::Result<Self> {
        // One reserve per grid slot, zero where nothing is committed. Upward
        // and downward reserves are summed apart: both are held, never netted.
        let mut reserves: Vec<PowerCommitment> = grid
            .iter()
            .map(|start_at| PowerCommitment {
                start_at,
                input_power: KiloWatt(0.0),
                output_power: KiloWatt(0.0),
            })
            .collect();
        for commitment in commitments {
            let reserve = &mut reserves[grid.index_of(&commitment.start_at)?];
            reserve.input_power += commitment.input_power;
            reserve.output_power += commitment.output_power;
        }
        Ok(Self {
            id,
            market,
            composition,
            commitments: reserves.try_into()?,
            penalty: ANCILLARY_PENALTY_EURO_PER_KW,
            activation_window,
        })
    }
    pub fn id(&self) -> &Uuid {
        &self.id
    }
    pub fn market(&self) -> &MarketSpecs {
        &self.market
    }
    pub fn commitments(&self) -> &TimeSeries<PowerCommitment> {
        &self.commitments
    }
    pub fn composition(&self) -> &[Uuid] {
        &self.composition
    }
    pub fn penalty(&self) -> &f64 {
        &self.penalty
    }
    pub fn activation_window(&self) -> &MinuteStep {
        &self.activation_window
    }
}
// endregion: Perimeter Definitions
