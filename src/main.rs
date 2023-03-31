mod package;
mod queries;
mod user;

use actix_cors::Cors;
use actix_web::{http, App, HttpServer};

use firestore::FirestoreDb;
use once_cell::sync::Lazy;

pub static DB: Lazy<FirestoreDb> =
    Lazy::new(|| futures::executor::block_on(async { init_database().await }));

const METADATA: &'static str = "metadata";

async fn init_database() -> FirestoreDb {
    FirestoreDb::new("ece-461-dev").await.unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("https://web.gcp.sammelson.com")
            .allowed_methods(vec![
                http::Method::DELETE,
                http::Method::GET,
                http::Method::POST,
                http::Method::PUT,
            ])
            .allowed_header("X-Authorization")
            .allowed_header("offset")
            .max_age(3600);

        App::new()
            .service(queries::authenticate)
            .service(queries::post_package)
            .service(queries::reset_registry)
            .service(queries::search_packages)
            .service(queries::get_rating_by_id)
            .service(queries::get_package_by_id)
            .service(queries::get_package_by_name)
            .service(queries::update_package_by_id)
            .service(queries::delete_package_by_id)
            .service(queries::get_package_by_regex)
            .service(queries::delete_package_by_name)
            .wrap(cors)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
