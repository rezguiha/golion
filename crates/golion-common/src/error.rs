use super::temporal::grid::TimeGridError;
use super::temporal::series::TimeSeriesError;
use derive_more::From;
#[derive(Debug, From)]
pub enum Error {
    #[from]
    Grid(TimeGridError),
    #[from]
    TimeSeries(TimeSeriesError),
}

pub type Result<T> = core::result::Result<T, Error>;
