/// Cycling and Throughput definitions.
use crate::temporal::series::{TimeSeries, TimeStampedUtc};
use crate::units::power::KiloWattHour;
use jiff::Timestamp;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DischargeThroughput {
    pub start_at: Timestamp,
    /// Cumulated Discharged Energy over step.
    pub discharged_energy: KiloWattHour,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BidirectionalThroughput {
    pub start_at: Timestamp,
    /// Cumulated Discharged Energy over step.
    pub discharged_energy: KiloWattHour,
    /// Cumulated Charged Energy over step.
    pub charged_energy: KiloWattHour,
}

impl TimeStampedUtc for BidirectionalThroughput {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}
impl TimeStampedUtc for DischargeThroughput {
    fn start_at(&self) -> &Timestamp {
        &self.start_at
    }
}

#[derive(Debug)]
pub enum ThroughputHistory {
    DischargeOnly(TimeSeries<DischargeThroughput>),
    Bidirectional(TimeSeries<BidirectionalThroughput>),
}

#[derive(Debug)]
pub struct Cycling {
    pub history: ThroughputHistory,
    pub max_24h_avg_cycles: f64,
    pub penalty: f64,
}
