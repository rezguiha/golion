use crate::{
    asset::bess::availability::Availability, temporal::series::TimeSeries,
    units::power::KiloWattHour,
};
// region: BessLimit
/// Bess state of charge operating range.
/// This is a limitation set by the asset operators.
#[derive(Debug)]
pub struct SocRange {
    pub min_soc: KiloWattHour,
    pub max_soc: KiloWattHour,
}
#[derive(Debug)]
pub struct BessLimits {
    pub soc_range: SocRange,
    pub availability: TimeSeries<Availability>,
}
// endregion: BessLimit
