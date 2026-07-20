/// Linear Approximated Certification curve representing
/// tradeoff between upward and downward bidding (f(upward)=downward).
/// It represents an envelope on ancillary.
use garde::Validate;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

// region: Certification Envelope
#[derive(Debug, Deserialize, Serialize, Validate, TypedBuilder)]
pub struct CertificationTradeoffLine {
    /// Maximum downward reserve when upward
    /// reserve is zero expressed in kW.
    #[garde(range(min = 0.0))]
    pub intercept_mw: f64,
    /// Represents change in upward vs downward.
    #[garde(skip)]
    pub slope: f64,
}

/// Approximated Maximum Certified values per directions.
#[derive(Debug, Deserialize, Serialize, Validate, TypedBuilder)]
pub struct CertifiedOperatingRange {
    #[garde(range(min = 0.0))]
    pub upward_power_mw: f64,
    #[garde(range(min = 0.0))]
    pub downward_power_mw: f64,
}
/// Certification Envelope to respect in optimization per asset.
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CertifiedEnvelope {
    CertifiedOperatingRange,
    CertificationTradeoffLine,
}
// endregion: Certification Envelope
