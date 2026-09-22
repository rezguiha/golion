use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use derive_more::From;
#[derive(Debug, From)]
pub enum ServerError {
    // --- External
    #[from]
    Contract(golion_contract::Error),
    #[from]
    Optimization(golion_optimization::Error),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::Contract(golion_contract::Error::UnsupportedAssetType { .. }) => {
                StatusCode::NOT_IMPLEMENTED
            }
            Self::Contract(_) | Self::Optimization(_) => StatusCode::BAD_REQUEST,
        };
        (status, format!("{self:?}")).into_response()
    }
}
