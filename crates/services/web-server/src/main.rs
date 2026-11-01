// region:      --- Modules
mod app;
mod config;
mod error;
mod routes;
// endregion:      --- Modules
use crate::config::APP_CONFIG;
use tracing::info;
use tracing_subscriber::EnvFilter;
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let app = app::create_app();
    // Expose on port 3000
    let listener =
        tokio::net::TcpListener::bind(format!("0.0.0.0:{}", APP_CONFIG.server_port))
            .await
            .unwrap();

    info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());

    axum::serve(listener, app).await.unwrap();
}
