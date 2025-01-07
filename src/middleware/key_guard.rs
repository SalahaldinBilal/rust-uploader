use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::helpers::get_env_value;

pub async fn key_guard(request: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    if request.method() == Method::GET {
        return Ok(next.run(request).await);
    }

    let key = request.headers().get("Key").ok_or(StatusCode::BAD_REQUEST);
    let key = key.unwrap().to_str().unwrap();

    if get_env_value("KEY") == key {
        return Ok(next.run(request).await);
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    }
}
