use super::temporal::grid::TimeGridError;
use super::temporal::series::TimeSeriesError;
use super::temporal::step::InvalidGranularity;
use super::units::efficiency::InvalidEfficiency;
use super::units::soc::InvalidSocFraction;
use derive_more::From;
#[derive(Debug, From)]
pub enum Error {
    // --- Temporal Errors
    #[from]
    Grid(TimeGridError),
    #[from]
    TimeSeries(TimeSeriesError),
    #[from]
    Step(InvalidGranularity),
    #[from]
    SOC(InvalidSocFraction),
    #[from]
    Efficiency(InvalidEfficiency),
}

pub type Result<T> = core::result::Result<T, Error>;
