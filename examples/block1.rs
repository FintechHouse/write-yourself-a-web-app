use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    // Initialize the router
    let app = Router::new().route("/", get(hello_world));
    println!("Server running on http://0.0.0.0:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Handler function for the root path
async fn hello_world() -> &'static str {
    "Hello, World!"
}
