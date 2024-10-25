use axum::{
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    routing::{any_service, MethodRouter},
};
use tower_http::services::ServeDir;

use crate::config;

pub fn serve_dir() -> MethodRouter {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "NOT FOUND")
    }

    any_service(ServeDir::new(&config().WEB_FOLDER).not_found_service(handle_404.into_service()))
}
