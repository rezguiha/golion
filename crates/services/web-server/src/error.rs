use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use derive_more::From;
#[derive(Debug, From)]
pub enum ServerError {
    // --- External
    #[from]
    Contract(lib_contract::Error),
    #[from]
    Optimization(lib_optimization::Error),
    #[from]
    SpawnedTask(tokio::task::JoinError),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::Contract(lib_contract::Error::UnsupportedAssetType { .. }) => {
                StatusCode::NOT_IMPLEMENTED
            }
            Self::Optimization(lib_optimization::Error::MipGap(_))
            | Self::Optimization(lib_optimization::Error::Solver(_))
            | Self::SpawnedTask(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Contract(_) | Self::Optimization(_) => StatusCode::BAD_REQUEST,
        };
        (status, format!("{self:?}")).into_response()
    }
}
