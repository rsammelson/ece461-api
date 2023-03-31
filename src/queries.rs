use crate::package::*;
use crate::user::*;
use crate::{DB, METADATA};

use axum::{
    extract::{Json, Path},
    response::IntoResponse,
};
use firestore::*;
use futures::{future, stream::BoxStream, StreamExt};
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

pub async fn post_package(Json(to_upload): Json<Package>) -> Result<impl IntoResponse, StatusCode> {
    // not yet implemeted:
    // 403: auth failed
    // 424: failed due to bad rating
    let prev_versions: BoxStream<PackageMetadata> = DB
        .fluent()
        .select()
        .from(METADATA)
        .filter(|q| {
            q.field(path!(PackageMetadata::name))
                .eq(&to_upload.metadata.name)
        })
        .obj()
        .stream_query()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if prev_versions
        .filter(|m| future::ready(m.version == to_upload.metadata.version))
        .count()
        .await
        >= 1
    {
        // 409: package already exists
        return Err(StatusCode::CONFLICT);
    }

    let id = PackageId::new();
    let _: PackageMetadata = DB
        .fluent()
        .insert()
        .into(METADATA)
        .document_id(id.to_string())
        .object(&to_upload.metadata)
        .execute()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 201: return package, with correct ID
    Ok(respond(
        StatusCode::CREATED,
        Package {
            metadata: PackageMetadata {
                id,
                ..to_upload.metadata
            },
            ..to_upload
        },
    ))
}

// TODO
pub async fn get_rating_by_id(Path(_id): Path<PackageId>) -> impl IntoResponse {
    // 200: return rating iff all rated
    // 404: does not exist
    // 500: package rating error
    ok(PackageRating::default())
}

/// Create an access token.
// TODO
pub async fn authenticate(Json(auth): Json<AuthenticationRequest>) -> impl IntoResponse {
    // 200: return token
    // 401: invalid user/password
    // 501: not implemented
    ok(AuthenticationToken::new(auth))
}

/// Return the history of this package (all versions).
// TODO
pub async fn get_package_by_name(Path(name): Path<String>) -> impl IntoResponse {
    // 200: return package history
    // 404: does not exist
    ok(vec![PackageHistoryEntry {
        metadata: PackageMetadata {
            name,
            ..Default::default()
        },
        ..Default::default()
    }])
}

/// Delete all versions of this package.
// TODO
pub async fn delete_package_by_name(Path(_name): Path<String>) -> impl IntoResponse {
    // 200: package deleted
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED
}

/// Get any packages fitting the regular expression.
///
/// Search for a package using regular expression over package names and READMEs.
// TODO
pub async fn get_package_by_regex(_regex: String) -> impl IntoResponse {
    // 200: return list of packages
    // 404: no packages found
    ok(vec![PackageMetadata::default()])
}
