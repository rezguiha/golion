mod app;
mod error;
mod routes;
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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());
    axum::serve(listener, app).await.unwrap();
}
