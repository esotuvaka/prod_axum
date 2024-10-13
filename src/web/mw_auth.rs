use crate::model::ModelController;
use crate::web::AUTH_TOKEN;
use crate::{Error, Result};
use axum::body::Body;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use tower_cookies::Cookies;

pub async fn require_auth(cookies: Cookies, req: Request<Body>, next: Next) -> Result<Response> {
    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    // TODO: Implement real auth token validation and parsing
    auth_token.ok_or(Error::AuthFailNoAuthTokenCookie)?;

    Ok(next.run(req).await)
}
