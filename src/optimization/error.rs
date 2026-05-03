use thiserror::Error;
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("No variable found for key : {key}")]
    MissingKey { key: String },
}
