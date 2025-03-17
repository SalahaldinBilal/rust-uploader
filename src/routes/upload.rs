use std::str::FromStr;
use std::sync::Arc;

use axum::{
    Extension,
    body::Body,
    extract::Path,
    http::{Request, StatusCode},
    response::Json,
};
use rand::distr::{Alphanumeric, SampleString};
use serde_json::{Value, json};
use std::collections::BTreeMap;

use crate::{
    AppState,
    helpers::{AxumBodyStreamWrapper, create_jwt_token, get_env_value, get_file_extension},
};

#[axum::debug_handler]
pub async fn write_file_to_b2(
    Extension(state): Extension<Arc<AppState>>,
    Path(file_name): Path<String>,
    request: Request<Body>,
) -> (StatusCode, Json<Value>) {
    let rand_id = Alphanumeric.sample_string(&mut rand::rng(), 10);

    let file_extension = get_file_extension(&file_name);
    let file_name = if file_extension.len() > 0 {
        format!("{rand_id}.{file_extension}")
    } else {
        rand_id
    };

    let file_size = match request.headers().get("content-length") {
        Some(value) => match u64::from_str(String::from_utf8_lossy(value.as_bytes()).as_ref()) {
            Ok(size) => size,
            Err(err) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(
                        json!({ "success": false, "err": format!("Failed to parse content length as number: {:#?}", err) }),
                    ),
                );
            }
        },
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "err": "Content Length header not specified." })),
            );
        }
    };

    let body = request.into_body();

    let stream = AxumBodyStreamWrapper::new(body.into_data_stream());

    let upload = state
        .b2_client
        .create_upload(
            stream,
            file_name,
            get_env_value("B2_BUCKET_ID"),
            None,
            file_size,
            None,
        )
        .await;

    let file_info = match upload.start().await {
        Ok(file) => file,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    json!({ "success": false, "err": format!("Failed to upload file: {:#?}", err) }),
                ),
            );
        }
    };

    let jwt_secret = get_env_value("JWT_SECRET");
    let token = create_jwt_token(
        &jwt_secret,
        &BTreeMap::from([
            ("filename", file_info.file_name.as_ref()),
            ("file_id", file_info.file_id.as_ref()),
        ]),
    )
    .expect("valid JWT token");

    let url = format!("{}/{}", get_env_value("IMAGE_URL"), file_info.file_name);

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
