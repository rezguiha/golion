/// Physical Variables definition.
/// It includes also their creation trait.
use golion_domain::temporal::series::TimeStampedUtc;
use good_lp::Variable;
use jiff::Timestamp;
// region: Bess Variables
/// Bess Variables container with time information
#[derive(Debug)]
pub struct BessVariables {
    pub start_at: Timestamp,
    pub input_power: Variable,
    pub output_power: Variable,
    pub soc: Variable,
}
// Implement TimeStampedUtc to enable creation
// of TimeSeries<BessVariables> out of Vec<BessVariables>.
impl TimeStampedUtc for BessVariables {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}

// endregion: Bess Variables
