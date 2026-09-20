use golion_domain::temporal::series::TimeStampedUtc;
use good_lp::Expression;
use jiff::Timestamp;
/// Bidding Variables container with time information
#[derive(Debug)]
pub struct BidVariables {
    pub(crate) start_at: Timestamp,
    pub(crate) input_power: Expression,
    pub(crate) output_power: Expression,
    pub(crate) input_energy: Expression,
    pub(crate) output_energy: Expression,
}
// Implement TimeStampedUtc to enable creation
// of TimeSeries<BidVariables> out of Vec<BidVariables>.
impl TimeStampedUtc for BidVariables {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}
