/// Time indexed series definition.
/// This will enable the single row access pattern on
/// containers that adhere to data oriented desin which have a time index
/// as a vec and other series as vec as well.
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use thiserror::Error;
pub trait HasTimeStamps {
    fn timestamps(&self) -> &[DateTime<Utc>];
}
/// Generic Wrapper over data that includes a mapping between
/// the timesteps
///
/// # Examples
///
/// ```
/// use golion::series::{IndexedSeries,HasTimeStamps};
/// use chrono::{DateTime,Utc};
/// struct ExampleTimeSeries{
///     start_at:Vec<DateTime<Utc>>,
///     value: Vec<f64>
/// }
/// impl HasTimeStamps for ExampleTimeSeries{
///  fn timestamps(&self)->&[DateTime<Utc>]{
/// &self.start_at
/// }
/// }
/// let ref_timestamp=Utc::now();
/// let example= ExampleTimeSeries{start_at:vec![ref_timestamp],value:vec![1.0]};
/// let indexed_example= IndexedSeries::<ExampleTimeSeries>::new(example).unwrap();
/// match indexed_example.index_at(&ref_timestamp){
/// Ok(result) => println!("Result: {result}"),
/// Err(msg) => println!("Error: {msg}"),
/// }
/// ```
#[derive(Debug)]
pub struct IndexedSeries<T> {
    pub data: T,
    /// Constructed mapping between datetime and the corresponding
    /// index in the vectors of the series contained in data attribute.
    index_by_time: HashMap<DateTime<Utc>, usize>,
}
/// Error definition for IndexedSeries Wrapper.
#[derive(Debug, Error)]
pub enum SeriesError {
    #[error("Duplicated timestamps found in index : {ts}")]
    DuplicatedTimeStamps { ts: DateTime<Utc> },
    #[error("Timestamp {ts} is missing from index.")]
    MissingTimeStamp { ts: DateTime<Utc> },
}
/// Constructor and
impl<T> IndexedSeries<T>
where
    T: HasTimeStamps,
{
    pub fn new(data: T) -> Result<Self, SeriesError> {
        let mut index_by_time: HashMap<DateTime<Utc>, usize> = HashMap::new();
        for (ind, ts) in data.timestamps().iter().copied().enumerate() {
            if index_by_time.insert(ts, ind).is_some() {
                return Err(SeriesError::DuplicatedTimeStamps { ts });
            };
        }
        Ok(Self { data, index_by_time })
    }
    pub fn index_at(&self, ts: &DateTime<Utc>) -> Result<&usize, SeriesError> {
        self.index_by_time.get(ts).ok_or(SeriesError::MissingTimeStamp { ts: *ts })
    }
}
