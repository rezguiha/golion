use crate::{
    asset::bess::availability::Availability, temporal::series::TimeSeries,
    units::soc::SocFraction,
};
// region: BessLimit
/// Bess state of charge operating range.
/// This is a limitation set by the asset operators.
#[derive(Debug)]
pub struct SocRange {
    pub min_soc: SocFraction,
    pub max_soc: SocFraction,
}
#[derive(Debug)]
pub struct BessLimits {
    pub soc_range: SocRange,
    pub availability: TimeSeries<Availability>,
}
// endregion: BessLimit
