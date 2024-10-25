// use axum::extract::Query;
use axum::response::{
    // IntoResponse,
    IntoResponse,
    Response,
};
use axum::routing::get_service;
use axum::{middleware, Json};
use axum::{
    // response::Html,
    Router,
};
use model::ModelController;
use serde_json::json;
// use serde::Deserialize;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use uuid::Uuid;

pub use self::error::{Error, Result};

mod ctx;
mod error;
mod log;
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
        .layer(middleware::from_fn_with_state(
            controller.clone(),
            web::mw_auth::ctx_resolver,
        ))
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
    let uuid = Uuid::new_v4();

    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "Error": {
                    "type": client_error.as_ref(),
                    "request_id": uuid.to_string(),
                }
            });
            println!("client_error_body: {client_error_body}");
            (*status_code, Json(client_error_body)).into_response()
        });

    println!("server log line - {uuid} - Error: {service_error:?}");
    println!();
    error_response.unwrap_or(res)
}

fn static_routes() -> Router {
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}
