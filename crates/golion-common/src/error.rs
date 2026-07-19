use super::temporal::grid::TimeGridError;
use super::temporal::series::TimeSeriesError;
use super::temporal::step::InvalidGranularity;
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
}

pub type Result<T> = core::result::Result<T, Error>;
