/// Defines a BESS asset specificationns.
/// These are the minimal attributes needed in input
/// to be able to model it.
use crate::{
    asset::bess::{
        availability::Availability,
        efficiency::BessPowerEfficiencies,
        soc::{SocBounds, SocRange, compute_soc_bounds},
        throughput::Cycling,
    },
    temporal::series::TimeSeries,
    units::power::{KiloWatt, KiloWattHour},
};

#[derive(Debug)]
pub struct BessSpecifications {
    pub soc_range: SocRange,
    pub availability: TimeSeries<Availability>,
    pub cycling: Cycling,
    pub efficiencies: BessPowerEfficiencies,
    pub rated_energy: KiloWattHour,
    pub rated_charge_power: KiloWatt,
    pub rated_discharge_power: KiloWatt,
}
impl BessSpecifications {
    pub fn efficiencies(&self) -> &BessPowerEfficiencies {
        &self.efficiencies
    }
    pub fn rated_usable_energy(&self) -> KiloWattHour {
        (self.rated_energy.0
            * (self.soc_range.max_soc().value() - self.soc_range.min_soc().value()))
        .into()
    }
    pub fn soc_bounds(&self) -> crate::Result<TimeSeries<SocBounds>> {
        compute_soc_bounds(&self.availability, &self.soc_range)
    }
}
