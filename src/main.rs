use axum::{routing::get, Router};
use tokio::net::TcpListener;

async fn health_check() -> &'static str {
    "Support Desk API berjalan!"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health_check));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server berjalan di http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
