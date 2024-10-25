mod config;
mod context;
mod error;
mod log;
mod model;
mod web;

pub use self::error::{Error, Result};
pub use config::config;

use axum::http::{Method, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::get_service;
use axum::Router;
use axum::{middleware, Json};
use context::ctx::Ctx;
use log::log::log_request;
use model::model::ModelController;
use serde_json::json;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;
use web::routes_static;

// Start server via:
// cargo watch -q -c -w src/ -x run
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let controller = ModelController::new().await?;

    // let routes_apis = web::routes_tickets::routes(controller.clone())
    //     .route_layer(middleware::from_fn(web::mw_auth::require_auth));

    let routes = Router::new()
        .merge(web::routes_login::routes())
        // .nest("/api", routes_apis)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            controller.clone(),
            web::mw_auth::ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static::serve_dir());

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("LISTENING on {addr}\n");
    axum::serve(listener, routes).await.unwrap();

    Ok(())
}

async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    debug!("{:<12} - main_response_mapper", "RES_MAPPER");
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
            debug!("client_error_body: {client_error_body}");
            (*status_code, Json(client_error_body)).into_response()
        });

    let client_error = client_status_error.unzip().1;
    let _ = log_request(uuid, req_method, uri, ctx, service_error, client_error).await;

    error_response.unwrap_or(res)
}
