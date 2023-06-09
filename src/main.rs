mod database;
mod queries;
mod scoring;
mod storage;
mod user;

#[cfg(feature = "log_request_response")]
mod log;

use queries::*;

use axum::{
    http::{header, HeaderName, HeaderValue, Method},
    routing::{delete, get, post, put},
    Router,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tower_http::{cors::CorsLayer, set_header::SetResponseHeaderLayer};

// single threaded runtime because expected to be run in a serverless, < 1 cpu environment
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cors_inner = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("https://web.gcp.sammelson.com"))
        .allow_headers([
            header::CONTENT_TYPE,
            HeaderName::from_static("offset"),
            // HeaderName::from_static("X-Authorization"),
        ])
        .expose_headers([HeaderName::from_static("offset")])
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT]);

    let cors = tower::ServiceBuilder::new()
        // apply cache header to CORS requests
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=604800"),
        ))
        .layer(cors_inner)
        // apply cache header to other requests
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ));

    let app = Router::new()
        .route("/package", post(queries::post_package))
        .route(
            "/package/:id",
            get(get_package_by_id)
                .put(update_package_by_id)
                .delete(delete_package_by_id),
        )
        .route("/packages", post(search_packages))
        .route("/package/:id/rate", get(get_rating_by_id))
        .route("/authenticate", put(authenticate))
        .route(
            "/package/byName/:name",
            get(get_package_by_name).delete(delete_package_by_name),
        )
        .route("/package/byRegEx", get(get_package_by_regex))
        .route("/reset", delete(reset_registry))
        .layer(cors);

    #[cfg(feature = "log_request_response")]
    let app = app.layer(axum::middleware::from_fn(log::print_request_response));

    axum::Server::bind(&SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        8080,
    ))
    .serve(app.into_make_service())
    .await?;

    Ok(())
}
