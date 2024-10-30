use crate::context::Ctx;
use crate::model::user::{UserController, UserLogin};
use crate::model::ModelManager;
use crate::web::{self, Error, Result};
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};
use tracing::debug;

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/api/login", post(api_login_handler))
        .with_state(mm)
}

async fn api_login_handler(
    State(mm): State<ModelManager>,
    cookies: Cookies,
    payload: Json<LoginPayload>,
) -> Result<Json<Value>> {
    debug!("{:<12} - api_login_handler", "HANDLER");

    let LoginPayload {
        username,
        pwd: pwd_clear,
    } = payload.0;
    let root_ctx = Ctx::root_ctx();

    let user: UserLogin = UserController::first_by_username(&root_ctx, &mm, &username)
        .await?
        .ok_or(Error::LoginFailUsernameNotFound)?; // WARN: DO NOT log the username, because users sometimes accidentally input their password
    let user_id = user.id;

    // FIXME: Implement real auth-token generation/signature.
    cookies.add(Cookie::new(web::AUTH_TOKEN, "user-1.exp.sign"));

    // Create the success body.
    let body = Json(json!({
        "result": {
            "success": true
        }
    }));

    Ok(body)
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    pwd: String,
}
