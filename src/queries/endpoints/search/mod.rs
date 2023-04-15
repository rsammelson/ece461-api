#[cfg(test)]
mod tests;

use super::{filter, ok, types::*, MyResponse};
use crate::database;

use axum::{
    extract::{Json, Query},
    http::{HeaderName, HeaderValue, StatusCode},
};
use firestore::{FirestoreQueryCursor, FirestoreQueryDirection, FirestoreValue};
use semver::VersionReq;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct SearchQuery {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Version")]
    pub version: Option<VersionReq>,
}

#[derive(Deserialize)]
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

    // hard to support this because of the firestore filtering / sorting rules
    //
    // ex:
    // search 1 is "name == abc, version == 123"
    // search 2 is "name == def, version >= 234"
    //
    // because of search 1, we *can not* tell firestore to sort by version
    // because of search 2, we *must* tell firestore to sort by version
    //
    // Possible solution is to make the requests one at a time, requesting again if the result
    // from the previous request didn't fill an entire page.
    //
    // Another possible solution is telling the client to do that themselves.
    // (the client knows when it's hit the last page because the offset header field isn't set)
    if search.len() > 1 {
        return Err(StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE);
    }

    // know at least one element from early return above
    let search = search.into_iter().next().unwrap();

    let db = database::get_database().await;

    let start = offset.and_then(|offs| Offset::parse(&offs));
    let show_all = search.name == "*";

    // if filtering with something like `version == "123"` directly, firestore can't sort by
    // version. However, we want to sort by version in other cases. This nonsense just detects if
    // the semver search is a direct equality, and pulls it out if so.
    //
    // Have to pull out just the one because if you filter by "inequality" (everything except
    // equality), firestore requires that you sort by that field first -- which we're not allowed
    // to do if also filtering that field by equality
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
        .fields(PACKAGE_METADATA_FIELDS)
        .from(database::METADATA)
        .limit(database::PAGE_LIMIT as u32);

    // filter out packages with names that don't match
    let query = query.filter(|q| {
        q.for_all([
            (!show_all)
                .then(|| q.field(database::NAME).eq(&search.name))
                .flatten(),
            version
                .as_ref()
                .and_then(|version| filter::versionreq_to_filter(&q, version)),
        ])
    });

    // because of firebase, can't sort by version if we require an eq comparator on version
    let query = if one_sort {
        query.order_by([(database::ID, FirestoreQueryDirection::Ascending)])
    } else {
        query.order_by([
            (database::VERSION, FirestoreQueryDirection::Ascending),
            (database::ID, FirestoreQueryDirection::Ascending),
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

    let result = query.obj().query().await.map_err(|e| {
        log::error!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 200: list of packages
    Ok(if result.len() < database::PAGE_LIMIT {
        // this is the last page, don't provide an offset for the next query
        ok(result)
    } else {
        let last_id = result
            .last()
            .map(|last| {
                HeaderValue::from_str(&format!("{},{}", last.version, last.id.as_ref())).unwrap()
            })
            .unwrap_or_else(|| HeaderValue::from_static(""));
        ok(result).push_header((HeaderName::from_static("offset"), last_id))
    })
}
