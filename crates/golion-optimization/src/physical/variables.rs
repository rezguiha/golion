use golion_domain::asset::bess::availability::Availability;
use golion_domain::asset::bess::limits::SocRange;
/// Physical Variables definition.
/// It includes also their creation trait.
use golion_domain::temporal::series::TimeStampedUtc;
use good_lp::{ProblemVariables, Variable, variable};
use jiff::Timestamp;
// region: Bess Variables
/// Bess Variables container with time information
#[derive(Debug)]
pub struct BessVariables {
    pub(crate) start_at: Timestamp,
    /// Represents active charge power in kW.
    pub(crate) input_power: Variable,
    /// Represents active discharge power in kW.
    pub(crate) output_power: Variable,
    /// Represents state of charge in kWh with starting
    /// interval convention.
    pub(crate) soc: Variable,
    /// Represents ancillary portion of charge (downward)
    /// commitments and bids that asset can deliver.
    pub(crate) input_ancillary: Variable,
    /// Represents ancillary portion of discharge (upward)
    /// commitments and bids that asset can deliver.
    pub(crate) output_ancillary: Variable,
}
impl BessVariables {
    pub fn try_new(
        dt: &Timestamp,
        avail_point: &Availability,
        soc_range: &SocRange,
        variable_generator: &mut ProblemVariables,
    ) -> crate::Result<Self> {
        Ok(Self {
            start_at: *dt,
            input_power: variable_generator
                .add(variable().min(0.0).max(avail_point.max_charge_power)),
            output_power: variable_generator
                .add(variable().min(0.0).max(avail_point.max_discharge_power)),
            soc: variable_generator.add(
                variable()
                    .min(
                        soc_range.min_soc.into_energy_kwh(&avail_point.max_usable_energy),
                    )
                    .max(
                        soc_range.max_soc.into_energy_kwh(&avail_point.max_usable_energy),
                    ),
            ),
            input_ancillary: variable_generator
                .add(variable().min(0.0).max(avail_point.max_charge_power)),
            output_ancillary: variable_generator
                .add(variable().min(0.0).max(avail_point.max_discharge_power)),
        })
    }
}
// Implement TimeStampedUtc to enable creation
// of TimeSeries<BessVariables> out of Vec<BessVariables>.
impl TimeStampedUtc for BessVariables {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}

// endregion: Bess Variables

// region: OtherAsset variables

/// This is a temporary implementation of variables
/// of types other than BESS.
#[derive(Debug)]
pub struct OtherAssetVariables {
    pub(crate) start_at: Timestamp,
    pub(crate) output_power: Variable,
}

// Implement TimeStampedUtc to enable creation
// of TimeSeries<BessVariables> out of Vec<BessVariables>.
impl TimeStampedUtc for OtherAssetVariables {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}

// endregion: OtherAsset variables
