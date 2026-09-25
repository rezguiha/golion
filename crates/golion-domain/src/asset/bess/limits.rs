use crate::{
    asset::bess::availability::Availability, temporal::series::TimeSeries,
    units::soc::SocFraction,
};
use derive_more::Display;
// region: BessLimit
#[derive(Debug, Display)]
#[display("Invalid soc range: min_soc {min_soc} is above max_soc {max_soc}.")]
pub struct InvalidSocRange {
    min_soc: f64,
    max_soc: f64,
}

/// Bess state of charge operating range.
#[derive(Debug)]
pub struct SocRange {
    min_soc: SocFraction,
    max_soc: SocFraction,
}

impl SocRange {
    pub fn try_new(min_soc: SocFraction, max_soc: SocFraction) -> crate::Result<Self> {
        if min_soc > max_soc {
            return Err(InvalidSocRange {
                min_soc: *min_soc.value(),
                max_soc: *max_soc.value(),
            }
            .into());
        }
        Ok(Self { min_soc, max_soc })
    }
    pub fn min_soc(&self) -> &SocFraction {
        &self.min_soc
    }
    pub fn max_soc(&self) -> &SocFraction {
        &self.max_soc
    }
}
#[derive(Debug)]
pub struct BessLimits {
    pub soc_range: SocRange,
    pub availability: TimeSeries<Availability>,
}
// endregion: BessLimit
