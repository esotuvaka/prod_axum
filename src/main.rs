// use axum::extract::Query;
use axum::middleware;
use axum::response::{
    // IntoResponse,
    Response,
};
use axum::routing::get_service;
use axum::{
    // response::Html,
    Router,
};
use model::ModelController;
// use serde::Deserialize;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;

pub use self::error::{Error, Result};

mod ctx;
mod error;
mod model;
mod web;

// Start server via:
// cargo watch -q -c -w src/ -x run
#[tokio::main]
async fn main() -> Result<()> {
    let controller = ModelController::new().await?;

    let routes_apis = web::routes_tickets::routes(controller.clone())
        .route_layer(middleware::from_fn(web::mw_auth::require_auth));

    let routes = Router::new()
        .merge(web::routes_login::routes())
        .nest("/api", routes_apis)
        .layer(middleware::map_response(main_response_mapper))
        .layer(CookieManagerLayer::new())
        .fallback_service(static_routes());

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("LISTENING on {addr}\n");
    axum::serve(listener, routes).await.unwrap();

    Ok(())
}

async fn main_response_mapper(res: Response) -> Response {
    println!("{:<12} - main_response_mapper", "RES_MAPPER");
    println!(); // Empty line between each request
    res
}

// #[derive(Debug, Deserialize)]
// struct HelloParams {
//     name: Option<String>,
// }

// async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
//     println!("{:<12} - handler_hello", "HANDLER");
//     let name = params.name.as_deref().unwrap_or("World!");
//     Html(format!("Hello <strong>{name}!!!</strong>"))
// }

fn static_routes() -> Router {
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}
