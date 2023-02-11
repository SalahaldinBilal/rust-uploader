mod helpers;
mod middleware;
mod routes;

use axum::routing::get;
use axum::Extension;
use axum::{extract::DefaultBodyLimit, middleware::from_fn, routing::post, Router};
use dotenv::dotenv;
use helpers::get_env_value;
// use middlewares;
use s3::{creds::Credentials, error::S3Error, Bucket};
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::limit::RequestBodyLimitLayer;

use crate::middleware::{key_guard::key_guard, route_log::request_logger};
use crate::routes::{delete::delete_file_from_b2, upload::write_file_to_b2};

#[derive(Clone)]
pub struct AppState {
    bucket_connection: Bucket,
}

#[tokio::main]
async fn main() -> Result<(), S3Error> {
    dotenv().ok();

    let bucket = Bucket::new(
        &get_env_value("B2_BUCKET_NAME"),
        s3::Region::Custom {
            region: get_env_value("B2_BUCKET_S3_REGION"),
            endpoint: get_env_value("B2_BUCKET_S3_ENDPOINT"),
        },
        Credentials::new(
            Some(&get_env_value("B2_KEY_ID")),
            Some(&get_env_value("B2_APP_KEY")),
            None,
            None,
            None,
        )?,
    )?;

    let app = Router::new()
        .route("/delete", get(delete_file_from_b2))
        .route("/upload", post(write_file_to_b2))
        .route_layer(from_fn(key_guard))
        .route_layer(from_fn(request_logger))
        .layer(
            ServiceBuilder::new()
                .layer(DefaultBodyLimit::disable())
                .layer(RequestBodyLimitLayer::new(
                    250 * 1024 * 1024, /* 250mb */
                ))
                .layer(tower_http::trace::TraceLayer::new_for_http())
                .layer(Extension(Arc::new(AppState {
                    bucket_connection: bucket,
                }))),
        );

    let address = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running at {}", address);
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
