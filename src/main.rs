use axum::extract::Query;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{response::Html, Router};
use serde::Deserialize;
use tokio::net::TcpListener;

// Start server via:
// cargo watch -q -c -w src/ -x run
#[tokio::main]
async fn main() {
    let routes_hello = Router::new().route("/hello", get(handler_hello));

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("LISTENING on {addr}\n");
    axum::serve(listener, routes_hello).await.unwrap();
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}

async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    println!("{:<12} - handler_hello", "HANDLER");
    let name = params.name.as_deref().unwrap_or("World!");
    Html(format!("Hello <strong>{name}!!!</strong>"))
}
