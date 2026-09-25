/// Defines a BESS asset specificationns.
/// These are the minimal attributes needed in input
/// to be able to model it.
use crate::{
    asset::bess::{efficiency::BessPowerEfficiencies, limits::BessLimits},
    units::power::{KiloWatt, KiloWattHour},
};

#[derive(Debug)]
pub struct BessSpecifications {
    pub limits: BessLimits,
    pub efficiencies: BessPowerEfficiencies,
    pub rated_energy: KiloWattHour,
    pub rated_charge_power: KiloWatt,
    pub rated_discharge_power: KiloWatt,
}
