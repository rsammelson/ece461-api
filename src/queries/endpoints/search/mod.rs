use super::{constants, filter, ok, MyResponse};
use crate::package::*;

use axum::{
    extract::{Json, Query},
    http::{HeaderName, HeaderValue, StatusCode},
};
use firestore::{FirestoreQueryCursor, FirestoreQueryDirection, FirestoreValue};
use semver::VersionReq;

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

    // know at least one element from early return above
    let search = search.into_iter().next().unwrap();

    let db = constants::get_database().await;

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

    let query = db
        .fluent()
        .select()
        .from(constants::METADATA)
        .limit(constants::PAGE_LIMIT as u32);

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

    // because of firebase, can't sort by version if we require an eq comparator on version
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
    Ok(if result.len() < constants::PAGE_LIMIT {
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
