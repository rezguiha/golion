// region:      --- Modules
mod app;
mod error;
mod routes;
// endregion:      --- Modules
use lib_core::_dev_utils;
use tracing::info;
use tracing_subscriber::EnvFilter;
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    // -- For DEV ONLY
    _dev_utils::init_dev().await;
    let app = app::create_app();
    // Expose on port 8080
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());

    axum::serve(listener, app).await.unwrap();
}
