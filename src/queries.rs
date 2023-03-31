use crate::{init_database, DB, METADATA};
use crate::{package::*, user::*};

use axum::{
    extract::{Json, Path},
    response::IntoResponse,
};
use http::{header, StatusCode};

/// helper function because can't do
/// `impl<T> IntoResponse for T where T: Serialize`
fn serialize(data: impl serde::Serialize) -> impl IntoResponse {
    serde_json::to_string(&data).unwrap()
}

/// helper function for constructing common use case of returning status ok with json body
fn ok(data: impl serde::Serialize) -> impl IntoResponse {
    respond(StatusCode::OK, data)
}

fn respond(code: StatusCode, data: impl serde::Serialize) -> impl IntoResponse {
    (
        code,
        [(header::CONTENT_TYPE, "application/json")],
        serialize(data),
    )
}

/// Get the packages from the registry.
///
/// Get any packages fitting the query. Search for packages satisfying the indicated query.
/// If you want to enumerate all packages, provide an array with a single PackageQuery whose name is "*".
/// The response is paginated; the response header includes the offset to use in the next query.
pub async fn search_packages(Json(search): Json<Vec<SearchQuery>>) -> impl IntoResponse {
    // 200: list of packages
    // 413: too many packages returned (shouldn't happen? it's paginated)
    ok(search
        .iter()
        .map(|_| PackageMetadata::default())
        .collect::<Vec<_>>())
}

/// Reset the registry
///
/// Reset the registry to a system default state.
pub async fn _reset_registry() -> impl IntoResponse {
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

/// Delete this version of the package.
pub async fn delete_package_by_id(Path(_id): Path<PackageId>) -> impl IntoResponse {
    // 200: package deleted
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED
}

pub async fn post_package(
    Json(Package { mut metadata, data }): Json<Package>,
) -> Result<impl IntoResponse, StatusCode> {
    // not yet implemeted:
    // 403: auth failed
    // 424: failed due to bad rating
    let db = DB.get_or_init(init_database).await;

    let prev_versions_count = db
        .fluent()
        .select()
        .from(METADATA)
        .filter(|q| {
            q.for_all([
                q.field("Name").eq(&metadata.name),
                q.field("Version").eq(&metadata.version),
            ])
        })
        .limit(1)
        .query()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .len();

    if prev_versions_count >= 1 {
        // 409: package already exists
        log::info!("Failing package upload due to at least one matching package");
        return Err(StatusCode::CONFLICT);
    }

    metadata.id = PackageId::new();

    db.fluent()
        .insert()
        .into(METADATA)
        .document_id(metadata.id.to_string())
        .object(&metadata)
        .execute()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    log::info!(
        "Successfully uploaded new package with id {}",
        metadata.id.to_string()
    );
    // 201: return package, with correct ID
    Ok(ok(Package { metadata, data }))
}

pub async fn get_rating_by_id(Path(_id): Path<PackageId>) -> impl IntoResponse {
    // 200: return rating iff all rated
    // 404: does not exist
    // 500: package rating error
    respond(StatusCode::NOT_IMPLEMENTED, PackageRating::default())
}

/// Create an access token.
pub async fn authenticate(Json(auth): Json<AuthenticationRequest>) -> impl IntoResponse {
    // 200: return token
    // 401: invalid user/password
    // 501: not implemented
    respond(StatusCode::NOT_IMPLEMENTED, AuthenticationToken::new(auth))
}

/// Return the history of this package (all versions).
pub async fn get_package_by_name(Path(name): Path<String>) -> impl IntoResponse {
    // 200: return package history
    // 404: does not exist
    respond(
        StatusCode::NOT_IMPLEMENTED,
        vec![PackageHistoryEntry {
            metadata: PackageMetadata {
                name,
                ..Default::default()
            },
            ..Default::default()
        }],
    )
}

/// Delete all versions of this package.
pub async fn delete_package_by_name(Path(_name): Path<String>) -> impl IntoResponse {
    // 200: package deleted
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED
}

/// Get any packages fitting the regular expression.
///
/// Search for a package using regular expression over package names and READMEs.
pub async fn get_package_by_regex(_regex: String) -> impl IntoResponse {
    // 200: return list of packages
    // 404: no packages found
    respond(
        StatusCode::NOT_IMPLEMENTED,
        vec![PackageMetadata::default()],
    )
}
