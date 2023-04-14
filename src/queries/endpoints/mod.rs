// #[cfg(test)]
// mod tests;

mod search;
pub use search::*;

use super::*;
use crate::{package::*, user::*};

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

/// Reset the registry
///
/// Reset the registry to a system default state.
pub async fn reset_registry() -> impl IntoResponse {
    // 200: reset registry
    // 401: not authorized
    StatusCode::NOT_IMPLEMENTED
}

/// Interact with the package with this ID
///
/// Return this package.
pub async fn get_package_by_id(Path(_id): Path<PackageId>) -> impl IntoResponse {
    // 200: return package
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED
}

/// Update the content of the package.
///
/// The name, version, and ID must match.
/// The package contents (from PackageData) will replace the previous contents.
pub async fn update_package_by_id(
    Path(_id): Path<PackageId>,
    Json(_info): Json<Package>,
) -> impl IntoResponse {
    // 200: package updated
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED
}

pub async fn post_package(
    Json(Package { mut metadata, data }): Json<Package>,
) -> Result<MyResponse<Package>, StatusCode> {
    // not yet implemeted:
    // 403: auth failed
    // 424: failed due to bad rating
    let db = constants::get_database().await;

    let prev_versions_count = db
        .fluent()
        .select()
        .from(constants::METADATA)
        .filter(|q| {
            q.for_all([
                q.field("Name").eq(&metadata.name),
                q.field("Version").eq(&metadata.version),
            ])
        })
        .limit(1)
        .query()
        .await
        .map_err(|e| {
            log::error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .len();

    if prev_versions_count >= 1 {
        // 409: package already exists
        log::info!("Failing package upload due to at least one matching package");
        return Err(StatusCode::CONFLICT);
    }

    metadata.id = PackageId::new();

    db.fluent()
        .insert()
        .into(constants::METADATA)
        .document_id(&metadata.id)
        .object(&metadata)
        .execute()
        .await
        .map_err(|e| {
            log::error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    log::info!(
        "Successfully uploaded new package with id {:?}",
        metadata.id
    );
    // 201: return package, with correct ID
    Ok(ok(Package { metadata, data }))
}

pub async fn get_rating_by_id(Path(_id): Path<PackageId>) -> impl IntoResponse {
    // 200: return rating iff all rated
    // 404: does not exist
    // 500: package rating error
    StatusCode::NOT_IMPLEMENTED
}

/// Delete this version of the package.
// not in baseline requirements
pub async fn delete_package_by_id(Path(_id): Path<PackageId>) -> impl IntoResponse {
    // 200: package deleted
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED
}

/// Create an access token.
// not in baseline requirements
pub async fn authenticate(
    Json(auth): Json<AuthenticationRequest>,
) -> MyResponse<AuthenticationToken> {
    // 200: return token
    // 401: invalid user/password
    // 501: not implemented
    respond(StatusCode::NOT_IMPLEMENTED, AuthenticationToken::new(auth))
}

/// Return the history of this package (all versions).
// not in baseline requirements
pub async fn get_package_by_name(Path(_name): Path<String>) -> impl IntoResponse {
    // 200: return package history
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED
}

/// Delete all versions of this package.
// not in baseline requirements
pub async fn delete_package_by_name(Path(_name): Path<String>) -> impl IntoResponse {
    // 200: package deleted
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED
}

/// Get any packages fitting the regular expression.
///
/// Search for a package using regular expression over package names and READMEs.
// not in baseline requirements
pub async fn get_package_by_regex(_regex: String) -> impl IntoResponse {
    // 200: return list of packages
    // 404: no packages found
    StatusCode::NOT_IMPLEMENTED
}
