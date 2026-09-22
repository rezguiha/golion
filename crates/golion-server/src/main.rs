mod app;
mod error;
mod routes;

#[tokio::main]
async fn main() {
    let app = app::create_app();
    // Expose on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running at http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
