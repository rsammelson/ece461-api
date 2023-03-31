mod package;
mod queries;
mod user;

use queries::*;

use axum::{
    http::{header, HeaderValue, Method},
    routing::{get, post, put},
    Router,
};
use firestore::FirestoreDb;
use http::HeaderName;
use once_cell::sync::Lazy;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tower_http::cors::CorsLayer;

pub static DB: Lazy<FirestoreDb> =
    Lazy::new(|| futures::executor::block_on(async { init_database().await }));

const METADATA: &'static str = "metadata";

async fn init_database() -> FirestoreDb {
    FirestoreDb::new("ece-461-dev").await.unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // build our application with a single route
    let app = Router::new()
        .route("/package", post(queries::post_package))
        .route(
            "/package/:id",
            get(get_package_by_id)
                .put(update_package_by_id)
                .delete(delete_package_by_id),
        )
        .route("/packages", post(search_packages))
        .route("/package/{id}/:rate", get(get_rating_by_id))
        .route("/authenticate", put(authenticate))
        .route(
            "/package/byName/:name",
            get(get_package_by_name).delete(delete_package_by_name),
        )
        .route("/package/byRegEx", get(get_package_by_regex))
        .layer(
            CorsLayer::new()
                .allow_origin("https://web.gcp.sammelson.com".parse::<HeaderValue>().unwrap())
                .allow_headers([header::CONTENT_TYPE, HeaderName::from_static("offset")])
                .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT]),
        );

    // run it with hyper on localhost:3000
    axum::Server::bind(&SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        8080,
    ))
    .serve(app.into_make_service())
    .await?;

    Ok(())
}
