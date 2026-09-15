use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use derive_more::From;
#[derive(Debug, From)]
pub enum ServerError {
    /// Gracefully handles unsupported asset types
    /// as solution only supports batteries for now.
    UnsupportedAssetType,
    // --- External
    #[from]
    Contract(golion_contract::Error),
    #[from]
    Domain(golion_domain::Error),
    #[from]
    Optimization(golion_optimization::Error),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::UnsupportedAssetType => StatusCode::NOT_IMPLEMENTED,
            Self::Contract(_) | Self::Domain(_) | Self::Optimization(_) => {
                StatusCode::BAD_REQUEST
            }
        };
        (status, format!("{self:?}")).into_response()
    }
}
