use crate::{
    asset::bess::availability::Availability,
    temporal::series::{TimeSeries, TimeStampedUtc},
    units::{power::KiloWattHour, soc::SocFraction},
};
use derive_more::Display;
use jiff::Timestamp;
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

impl BessLimits {
    /// State of charge bounds per slot: the soc range applied to the
    /// usable energy available over the slot.
    pub fn soc_bounds(&self) -> crate::Result<TimeSeries<SocBounds>> {
        self.availability
            .data()
            .iter()
            .map(|avail_point| SocBounds {
                start_at: avail_point.start_at,
                min: self
                    .soc_range
                    .min_soc()
                    .into_energy_kwh(&avail_point.max_usable_energy),
                max: self
                    .soc_range
                    .max_soc()
                    .into_energy_kwh(&avail_point.max_usable_energy),
            })
            .collect::<Vec<_>>()
            .try_into()
    }
}
// endregion: BessLimit

// region: SocBounds
/// State of charge allowed over a slot, in kWh.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SocBounds {
    pub start_at: Timestamp,
    pub min: KiloWattHour,
    pub max: KiloWattHour,
}
impl TimeStampedUtc for SocBounds {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}
// endregion: SocBounds
