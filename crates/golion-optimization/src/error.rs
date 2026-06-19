use thiserror::Error;
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("No variable found for key : {key}")]
    MissingKey { key: String },
}

#[derive(Debug, Error)]
pub enum SeriesError {
    #[error("No data found for {entity} at index : {index}")]
    MissingValue { entity: String, index: usize },
    #[error(
        "{entity} has length {length} which differs from reference {reference_length}"
    )]
    MismatchedLength { entity: String, length: usize, reference_length: usize },
}
