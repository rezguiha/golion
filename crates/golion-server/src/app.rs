use crate::routes;
use axum::Router;
pub fn create_app() -> Router {
    Router::new().merge(routes::optimize::router())
}
