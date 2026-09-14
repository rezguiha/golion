use crate::temporal::series::TimeStampedUtc;
use jiff::Timestamp;
#[derive(Debug)]
pub struct RevenuePerKiloWatt {
    pub start_at: Timestamp,
    pub input_revenue: f64,
    pub output_revenue: f64,
}
impl TimeStampedUtc for RevenuePerKiloWatt {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}
#[derive(Debug)]
pub struct RevenuePerKiloWattHour {
    pub start_at: Timestamp,
    pub input_revenue: f64,
    pub output_revenue: f64,
}
impl TimeStampedUtc for RevenuePerKiloWattHour {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}
