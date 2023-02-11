use std::sync::Arc;

use axum::{extract::Query, http::StatusCode, response::Json, Extension};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    helpers::{get_env_value, verify_jwt_token},
    AppState,
};

#[derive(Deserialize)]
pub struct DeletionToken {
    token_str: String,
}

pub async fn delete_file_from_b2(
    Extension(state): Extension<Arc<AppState>>,
    Query(query): Query<DeletionToken>,
) -> (StatusCode, Json<Value>) {
    let token = query.token_str;

    if token.len() <= 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Please provide the JWT token." })),
        );
    }

    let claims = verify_jwt_token(&get_env_value("JWT_SECRET"), &token);

    if let Err(error) = claims {
        let error_message = format!("Failed to parse JWT token, error: {}", error.to_string());
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "message": error_message })),
        );
    }

    let claims = claims.unwrap();
    let filename = claims.get("filename").unwrap();

    let result = state.bucket_connection.delete_object(filename).await;

    if let Err(error) = result {
        let error_message = format!("Failed to delete file, error: {}", error.to_string());
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "message": error_message })),
        );
    }

    (
        StatusCode::OK,
        Json(json!({ "success": true, "message": "File deleted successfully." })),
    )
}
