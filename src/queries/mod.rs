#[cfg(test)]
mod tests;

mod constants;

use crate::{package::*, user::*};
use constants::{get_database, METADATA};

use axum::{
    extract::{Json, Path},
    response::IntoResponse,
};
use http::{header, HeaderMap, HeaderValue, StatusCode};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct MyResponse {
    code: StatusCode,
    headers: Vec<(header::HeaderName, HeaderValue)>,
    body: String,
}

impl IntoResponse for MyResponse {
    fn into_response(self) -> axum::response::Response {
        let headers = HeaderMap::from_iter(self.headers.into_iter());
        (self.code, headers, self.body).into_response()
    }
}

impl From<StatusCode> for MyResponse {
    fn from(value: StatusCode) -> Self {
        MyResponse {
            code: value,
            ..Default::default()
        }
    }
}

/// helper function because can't do
/// `impl<T> IntoResponse for T where T: Serialize`
fn serialize(data: impl serde::Serialize) -> String {
    serde_json::to_string(&data).unwrap()
}

/// helper function for constructing common use case of returning status ok with json body
fn ok(data: impl serde::Serialize) -> MyResponse {
    respond(StatusCode::OK, data)
}

fn respond(code: StatusCode, data: impl serde::Serialize) -> MyResponse {
    MyResponse {
        code,
        headers: vec![(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )],
        body: serialize(data),
    }
}

/// Get the packages from the registry.
///
/// Get any packages fitting the query. Search for packages satisfying the indicated query.
/// If you want to enumerate all packages, provide an array with a single PackageQuery whose name is "*".
/// The response is paginated; the response header includes the offset to use in the next query.
pub async fn search_packages(
    Json(search): Json<Vec<SearchQuery>>,
) -> Result<MyResponse, StatusCode> {
    // 413: too many packages returned (shouldn't happen? it's paginated)
    let db = get_database().await;

    let result = futures::future::join_all(search.iter().map(|query| async {
        let query = db
            .fluent()
            .select()
            .from(METADATA)
            .filter(|q| {
                q.for_all([
                    q.field("Name").eq(&query.name),
                    query
                        .version
                        .as_ref()
                        .and_then(|v| q.field("Version").eq(v)),
                ])
            })
            .obj()
            .query()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Result::<Vec<PackageMetadata>, StatusCode>::Ok(query)
    }))
    .await
    .into_iter()
    .collect::<Result<Vec<Vec<_>>, _>>()?
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    // 200: list of packages
    Ok(ok(result))
}

/// Reset the registry
///
/// Reset the registry to a system default state.
pub async fn reset_registry() -> MyResponse {
    // 200: reset registry
    // 401: not authorized
    StatusCode::NOT_IMPLEMENTED.into()
}

/// Interact with the package with this ID
///
/// Return this package.
pub async fn get_package_by_id(Path(_id): Path<PackageId>) -> MyResponse {
    // 200: return package
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED.into()
}

/// Update the content of the package.
///
/// The name, version, and ID must match.
/// The package contents (from PackageData) will replace the previous contents.
pub async fn update_package_by_id(
    Path(_id): Path<PackageId>,
    Json(_info): Json<Package>,
) -> MyResponse {
    // 200: package updated
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED.into()
}

/// Delete this version of the package.
pub async fn delete_package_by_id(Path(_id): Path<PackageId>) -> MyResponse {
    // 200: package deleted
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED.into()
}

pub async fn post_package(
    Json(Package { mut metadata, data }): Json<Package>,
) -> Result<MyResponse, StatusCode> {
    // not yet implemeted:
    // 403: auth failed
    // 424: failed due to bad rating
    let db = get_database().await;

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

pub async fn get_rating_by_id(Path(_id): Path<PackageId>) -> MyResponse {
    // 200: return rating iff all rated
    // 404: does not exist
    // 500: package rating error
    respond(StatusCode::NOT_IMPLEMENTED, PackageRating::default())
}

/// Create an access token.
pub async fn authenticate(Json(auth): Json<AuthenticationRequest>) -> MyResponse {
    // 200: return token
    // 401: invalid user/password
    // 501: not implemented
    respond(StatusCode::NOT_IMPLEMENTED, AuthenticationToken::new(auth))
}

/// Return the history of this package (all versions).
pub async fn get_package_by_name(Path(name): Path<String>) -> MyResponse {
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
pub async fn delete_package_by_name(Path(_name): Path<String>) -> MyResponse {
    // 200: package deleted
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED.into()
}

/// Get any packages fitting the regular expression.
///
/// Search for a package using regular expression over package names and READMEs.
pub async fn get_package_by_regex(_regex: String) -> MyResponse {
    // 200: return list of packages
    // 404: no packages found
    respond(
        StatusCode::NOT_IMPLEMENTED,
        vec![PackageMetadata::default()],
    )
}