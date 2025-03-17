use std::sync::Arc;

use axum::{Extension, extract::Query, http::StatusCode, response::Json};
use backblaze_b2_client::definitions::bodies::B2DeleteFileVersionBody;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    AppState,
    helpers::{get_env_value, verify_jwt_token},
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

    let claims = match verify_jwt_token(&get_env_value("JWT_SECRET"), &token) {
        Ok(c) => c,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    json!({ "success": false, "message": format!("Failed to parse JWT token, error: {:#?}", error) }),
                ),
            );
        }
    };

    let file_id = match claims.get("file_id") {
        Some(id) => id.clone(),
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Claims Missing 'file_id'" })),
            );
        }
    };
    let filename = match claims.get("filename") {
        Some(id) => id.clone(),
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Claims Missing 'filename'" })),
            );
        }
    };

    let body = B2DeleteFileVersionBody::builder()
        .file_name(filename)
        .file_id(file_id)
        .build();

    match state
        .b2_client
        .basic_client()
        .delete_file_version(body)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "success": true, "message": "File deleted successfully." })),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                json!({ "success": false, "message": format!("Failed to delete file, error: {:#?}", err) }),
            ),
        ),
    }
}
