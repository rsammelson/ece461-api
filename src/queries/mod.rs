#[cfg(test)]
mod tests;

mod constants;
use constants::{get_database, METADATA, PAGE_LIMIT};

use crate::{package::*, user::*};

use axum::{
    extract::{Json, Path, Query},
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::IntoResponse,
};
use firestore::{FirestoreQueryCursor, FirestoreQueryDirection, FirestoreValue};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct MyResponse<T> {
    code: StatusCode,
    headers: Vec<(HeaderName, HeaderValue)>,
    body: T,
}

impl<T> MyResponse<T> {
    fn push_header(mut self, header: (HeaderName, HeaderValue)) -> Self {
        self.headers.push(header);
        self
    }
}

impl<T> IntoResponse for MyResponse<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let headers = HeaderMap::from_iter(self.headers.into_iter());
        (
            self.code,
            headers,
            serde_json::to_string(&self.body).unwrap_or_default(),
        )
            .into_response()
    }
}

/// helper function for constructing common use case of returning status ok with json body
fn ok<T: serde::Serialize>(body: T) -> MyResponse<T> {
    respond(StatusCode::OK, body)
}

fn respond<T: serde::Serialize>(code: StatusCode, body: T) -> MyResponse<T> {
    MyResponse {
        code,
        headers: vec![(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )],
        body,
    }
}

#[derive(serde::Deserialize)]
pub struct Offset {
    offset: Option<PackageId>,
}

/// Get the packages from the registry.
///
/// Get any packages fitting the query. Search for packages satisfying the indicated query.
/// If you want to enumerate all packages, provide an array with a single PackageQuery whose name is "*".
/// The response is paginated; the response header includes the offset to use in the next query.
pub async fn search_packages(
    Query(Offset { offset }): Query<Offset>,
    Json(search): Json<Vec<SearchQuery>>,
) -> Result<MyResponse<Vec<PackageMetadata>>, StatusCode> {
    let db = get_database().await;

    let start: Option<FirestoreValue> = offset.map(|id| id.into());
    let show_all = search.len() == 1 && search[0].name == "*";

    let query = db.fluent().select().from(METADATA);
    let query = if show_all {
        query
    } else {
        // TODO: figure out how to do version matching? can't just be a simple string comparison,
        // since they want semver. If we match after the fact, can't necessarily use the
        // `.limit(PAGE_LIMIT)`, as we might filter out too many.
        //
        // Maybe just make another request if version filters out too many?
        query.filter(|q| {
            q.field("Name")
                .is_in(search.iter().map(|s| &s.name).collect::<Vec<_>>())
        })
    };
    let query = match start {
        Some(start) => query.start_at(FirestoreQueryCursor::AfterValue(vec![start])),
        None => query,
    }
    .order_by([("ID", FirestoreQueryDirection::Descending)])
    .limit(PAGE_LIMIT);

    let result: Vec<PackageMetadata> = query.obj().query().await.map_err(|e| {
        log::error!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 200: list of packages
    Ok(if result.len() < PAGE_LIMIT as usize {
        // this is the last page, don't provide an offset for the next query
        ok(result)
    } else {
        let last_id = result.last().unwrap().id.parse().unwrap();
        // this isn't the last page
        ok(result).push_header((HeaderName::from_static("offset"), last_id))
    })
}

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

/// Delete this version of the package.
pub async fn delete_package_by_id(Path(_id): Path<PackageId>) -> impl IntoResponse {
    // 200: package deleted
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED
}

pub async fn post_package(
    Json(Package { mut metadata, data }): Json<Package>,
) -> Result<MyResponse<Package>, StatusCode> {
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
        .into(METADATA)
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

pub async fn get_rating_by_id(Path(_id): Path<PackageId>) -> MyResponse<PackageRating> {
    // 200: return rating iff all rated
    // 404: does not exist
    // 500: package rating error
    respond(StatusCode::NOT_IMPLEMENTED, PackageRating::default())
}

/// Create an access token.
pub async fn authenticate(
    Json(auth): Json<AuthenticationRequest>,
) -> MyResponse<AuthenticationToken> {
    // 200: return token
    // 401: invalid user/password
    // 501: not implemented
    respond(StatusCode::NOT_IMPLEMENTED, AuthenticationToken::new(auth))
}

/// Return the history of this package (all versions).
pub async fn get_package_by_name(Path(name): Path<String>) -> MyResponse<Vec<PackageHistoryEntry>> {
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
pub async fn get_package_by_regex(_regex: String) -> MyResponse<Vec<PackageMetadata>> {
    // 200: return list of packages
    // 404: no packages found
    respond(
        StatusCode::NOT_IMPLEMENTED,
        vec![PackageMetadata::default()],
    )
}
