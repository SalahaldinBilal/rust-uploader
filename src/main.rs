mod helpers;
mod macros;
mod middleware;
mod routes;

use axum::Extension;
use axum::routing::get;
use axum::{Router, extract::DefaultBodyLimit, middleware::from_fn, routing::post};
use backblaze_b2_client::client::B2Client;
use backblaze_b2_client::error::B2Error;
use backblaze_b2_client::util::SizeUnit;
use dotenvy::dotenv;
use helpers::get_env_value;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::limit::RequestBodyLimitLayer;

use crate::middleware::{key_guard::key_guard, route_log::request_logger};
use crate::routes::{delete::delete_file_from_b2, upload::write_file_to_b2};

#[derive(Clone)]
pub struct AppState {
    b2_client: Arc<B2Client>,
}

#[tokio::main]
async fn main() -> Result<(), B2Error> {
    dotenv().ok();

    let key_id = get_env_value("B2_KEY_ID");
    let application_key = get_env_value("B2_APP_KEY");
    get_env_value("B2_BUCKET_ID");
    get_env_value("JWT_SECRET");
    get_env_value("API_URL");
    get_env_value("KEY");
    get_env_value("IMAGE_URL");

    let client = B2Client::new(key_id, application_key).await?;

    let app = Router::new()
        .route("/delete", get(delete_file_from_b2))
        .route("/upload/{file_name}", post(write_file_to_b2))
        .route_layer(from_fn(key_guard))
        .route_layer(from_fn(request_logger))
        .layer(
            ServiceBuilder::new()
                .layer(DefaultBodyLimit::disable())
                .layer(RequestBodyLimitLayer::new(
                    (SizeUnit::GIBIBYTE * 10) as usize,
                ))
                .layer(tower_http::trace::TraceLayer::new_for_http())
                .layer(Extension(Arc::new(AppState {
                    b2_client: Arc::new(client),
                }))),
        );

    let address = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running at {}", address);

    let listener = TcpListener::bind(address).await.expect("valid listener");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
