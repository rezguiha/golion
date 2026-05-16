use axum::{Json, Router, routing::post};
use golion_domain::asset::core::AssetData;
/// Temporary handler that prints payload to make sure
/// at least deserialization is working as expected.
async fn handler_optimize(payload: Json<AssetData>) {
    println!("{}", serde_json::to_string_pretty(&payload.0).unwrap());
}

#[tokio::main]
async fn main() {
    // Create route for optimization run
    let app = Router::new().route("/optimize", post(handler_optimize));
    // Expose on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running at http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
