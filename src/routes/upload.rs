use std::sync::Arc;

use axum::{extract::Multipart, http::StatusCode, response::Json, Extension};
use rand::distributions::{Alphanumeric, DistString};
use serde_json::{json, Value};
use std::collections::BTreeMap;

use crate::{
    helpers::{create_jwt_token, get_env_value, get_file_extension},
    AppState,
};

pub async fn write_file_to_b2(
    Extension(state): Extension<Arc<AppState>>,
    mut multipart: Multipart,
) -> (StatusCode, Json<Value>) {
    let rand_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 10);
    let field_option = multipart.next_field().await.unwrap();

    if let None = field_option {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false })));
    }

    let field = field_option.unwrap();
    let field_name = field.name().unwrap().to_string();

    if field_name != "file" {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false })));
    }

    let file_name = field.file_name().unwrap().to_string();
    let file_extension = get_file_extension(&file_name);
    let file_mimetype = field.content_type().unwrap().to_string();
    let temp_file_name = if file_extension.len() > 0 {
        format!("{rand_id}.{file_extension}")
    } else {
        rand_id
    };

    let data = field.bytes().await.unwrap();

    let upload_result = state
        .bucket_connection
        .put_object_with_content_type(&temp_file_name, &data, &file_mimetype)
        .await;

    if let Err(_) = upload_result {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false })));
    }

    let jwt_secret = get_env_value("JWT_SECRET");
    let token = create_jwt_token(
        &jwt_secret,
        &BTreeMap::from([("filename", temp_file_name.as_str())]),
    );
    let url = format!("{}/{}", get_env_value("IMAGE_URL"), temp_file_name);
    let deletion = format!(
        "{}/delete?token_str={}",
        get_env_value("API_URL"),
        token.unwrap()
    );

    let final_json = Json(json!({
        "success": true,
        "data": {
            "url": url,
            "deletion": deletion
        }
    }));

    (StatusCode::OK, final_json)
}
