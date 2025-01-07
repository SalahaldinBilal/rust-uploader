use std::sync::Arc;

use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{Json, Response},
    Extension,
};
use rand::distributions::{Alphanumeric, DistString};
use serde_json::{json, Value};
use std::collections::BTreeMap;

use crate::{
    helpers::{create_jwt_token, get_env_value, get_file_extension},
    simple_response, AppState,
};

pub async fn write_file_to_b2(
    Extension(state): Extension<Arc<AppState>>,
    mut multipart: Multipart,
) -> (StatusCode, Json<Value>) {
    let rand_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 10);
    let field_option = match multipart.next_field().await {
        Ok(field) => field,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "err": "Need at least one file" })),
            )
        }
    };

    let field = match field_option {
        Some(field) => field,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "err": "Need at least one file" })),
            )
        }
    };

    let field_name = match field.name() {
        Some(name) => name,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "err": "Can't find file with key 'file'" })),
            )
        }
    };

    if field_name != "file" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "err": "Can't find file with key 'file'" })),
        );
    }

    let file_name = match field.file_name() {
        Some(name) => name,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "err": "'file' needs to have a name" })),
            )
        }
    };

    let file_mimetype = match field.content_type() {
        Some(name) => name.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    json!({ "success": false, "err": "'file' needs to have proper content type" }),
                ),
            )
        }
    };

    let file_extension = get_file_extension(file_name);
    let temp_file_name = if file_extension.len() > 0 {
        format!("{rand_id}.{file_extension}")
    } else {
        rand_id
    };

    let data = match field.bytes().await {
        Ok(bytes) => bytes,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    json!({ "success": false, "err": format!("Failed to read file: {}", err.to_string()) }),
                ),
            )
        }
    };

    let upload_result = state
        .bucket_connection
        .put_object_with_content_type(&temp_file_name, &data, &file_mimetype)
        .await;

    if let Err(err) = upload_result {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                json!({ "success": false, "err": format!("Failed to upload file: {}", err.to_string()) }),
            ),
        );
    }

    let jwt_secret = get_env_value("JWT_SECRET");
    let token = create_jwt_token(
        &jwt_secret,
        &BTreeMap::from([("filename", temp_file_name.as_str())]),
    )
    .expect("valid JWT token");

    let url = format!("{}/{}", get_env_value("IMAGE_URL"), temp_file_name);

    let deletion = format!("{}/delete?token_str={}", get_env_value("API_URL"), token);

    let final_json = Json(json!({
        "success": true,
        "data": {
            "url": url,
            "deletion": deletion
        }
    }));

    (StatusCode::OK, final_json)
}
