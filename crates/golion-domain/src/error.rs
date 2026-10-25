use super::asset::bess::soc::InvalidSocRange;
use super::temporal::grid::TimeGridError;
use super::temporal::series::TimeSeriesError;
use super::temporal::step::StepError;
use super::units::efficiency::InvalidEfficiency;
use super::units::soc::InvalidSocFraction;
use jiff::Error as JiffError;

use super::market::error::MarketError;
use super::problem::error::ProblemError;
use derive_more::From;
#[derive(Debug, From)]
pub enum Error {
    // --- Temporal Errors
    #[from]
    Grid(TimeGridError),
    #[from]
    TimeSeries(TimeSeriesError),
    #[from]
    Step(StepError),
    // -- Unit Errors
    #[from]
    SOC(InvalidSocFraction),
    #[from]
    Efficiency(InvalidEfficiency),
    // -- Asset Errors
    #[from]
    SocRange(InvalidSocRange),
    // -- Market Errors
    #[from]
    Market(MarketError),
    // -- Problem Errors
    #[from]
    Problem(ProblemError),
    // -- External
    #[from]
    JiffConversionError(JiffError),
}

pub type Result<T> = core::result::Result<T, Error>;
