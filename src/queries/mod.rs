#[cfg(test)]
mod tests;

mod constants;
mod filter;
use constants::{get_database, METADATA, PAGE_LIMIT};
use semver::VersionReq;

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
    offset: Option<String>,
}

impl Offset {
    fn parse(offset: &str) -> Option<Vec<FirestoreValue>> {
        offset
            .split_once(',')
            .map(|(v, i)| vec![v.into(), i.into()])
    }
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
    // have to have packages to search for
    if search.len() < 1 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // don't want to support this
    if search.len() > 1 {
        return Err(StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE);
    }

    let search = search.into_iter().next().unwrap();

    let db = get_database().await;

    let start = offset.and_then(|offs| Offset::parse(&offs));
    let show_all = search.name == "*";

    let requires_eq = search
        .version
        .as_ref()
        .map(|v| {
            v.comparators
                .iter()
                .find_map(|c| filter::comparator_requires_eq(c).then_some(c.clone()))
        })
        .flatten();
    let one_sort = requires_eq.is_some();
    let version = match requires_eq {
        Some(requires_eq) => Some(VersionReq {
            comparators: vec![requires_eq],
        }),
        None => search.version,
    };

    let query = db.fluent().select().from(METADATA).limit(PAGE_LIMIT as u32);

    // filter out packages with names that don't match
    let query = if show_all {
        query.filter(|q| q.for_all(filter::versionreq_to_filter(&q, version.as_ref()?)))
    } else {
        query.filter(|q| {
            q.for_all([
                q.field("Name").eq(&search.name),
                version
                    .as_ref()
                    .and_then(|version| filter::versionreq_to_filter(&q, version)),
            ])
        })
    };

    let query = if one_sort {
        query.order_by([("ID", FirestoreQueryDirection::Ascending)])
    } else {
        query.order_by([
            ("Version", FirestoreQueryDirection::Ascending),
            ("ID", FirestoreQueryDirection::Ascending),
        ])
    };

    // start at the offset given
    let query = match start {
        Some(start) => query.start_at(FirestoreQueryCursor::AfterValue(if one_sort {
            start.into_iter().skip(1).collect()
        } else {
            start
        })),
        None => query,
    };

    // get a set of packages that have names that match
    let result: Vec<PackageMetadata> = query.obj().query().await.map_err(|e| {
        log::error!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 200: list of packages
    Ok(if result.len() < PAGE_LIMIT {
        // this is the last page, don't provide an offset for the next query
        ok(result)
    } else {
        let last_id = result
            .last()
            .map(|last| {
                HeaderValue::from_str(&format!("{},{}", last.version, last.id.as_ref())).unwrap()
            })
            .unwrap_or_else(|| HeaderValue::from_static(""));
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

pub async fn get_rating_by_id(Path(_id): Path<PackageId>) -> impl IntoResponse {
    // 200: return rating iff all rated
    // 404: does not exist
    // 500: package rating error
    StatusCode::NOT_IMPLEMENTED
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
pub async fn get_package_by_name(Path(_name): Path<String>) -> impl IntoResponse {
    // 200: return package history
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED
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
    StatusCode::NOT_IMPLEMENTED
}
