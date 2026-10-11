/// Defines a BESS asset specificationns.
/// These are the minimal attributes needed in input
/// to be able to model it.
use crate::{
    asset::bess::{
        efficiency::BessPowerEfficiencies, limits::BessLimits, throughput::Cycling,
    },
    units::power::{KiloWatt, KiloWattHour},
};

#[derive(Debug)]
pub struct BessSpecifications {
    pub limits: BessLimits,
    pub cycling: Cycling,
    pub efficiencies: BessPowerEfficiencies,
    pub rated_energy: KiloWattHour,
    pub rated_charge_power: KiloWatt,
    pub rated_discharge_power: KiloWatt,
}

impl BessSpecifications {
    fn usable_energy(&self) -> KiloWattHour {
        (self.rated_energy.0
            * (self.limits.soc_range.max_soc().value()
                - self.limits.soc_range.min_soc().value()))
        .into()
    }
}
