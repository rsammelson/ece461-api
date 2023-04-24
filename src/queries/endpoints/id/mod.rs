use firestore::FirestoreDb;

use super::{ok, types::*, MyResponse};
use crate::{
    database::{self, DatabaseEntry},
    scoring::{self, RatedPackage, RatingError},
};

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

async fn find_package_by_id<F, T>(
    db: FirestoreDb,
    id: PackageId,
    fields: F,
) -> Result<T, StatusCode>
where
    F: IntoIterator,
    F::Item: AsRef<str>,
    T: Send + for<'de> serde::Deserialize<'de>,
{
    let query = db
        .fluent()
        .select()
        .fields(fields)
        .from(database::METADATA)
        .limit(1)
        .filter(|q| q.field(database::ID).eq(&id));

    let query_result = query.obj().query().await.map_err(|e| {
        log::error!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match query_result.into_iter().next() {
        Some(r) => Ok(r),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Interact with the package with this ID
///
/// Return this package.
pub async fn get_package_by_id(
    Path(id): Path<PackageId>,
) -> Result<MyResponse<Package>, StatusCode> {
    // 200: return package
    // 404: does not exist
    // note: this doesn't actually meet the spec
    // they want to have the content when downloading, we're giving a url
    // exodus is expensive from api, cheap from obj storage
    // they can just download it directly from there
    let db = database::get_database().await;
    match find_package_by_id(db, id, PACKAGE_FIELDS).await? {
        Some(package) => Ok(ok(package)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Update the content of the package.
///
/// The name, version, and ID must match.
/// The package contents (from PackageData) will replace the previous contents.
pub async fn update_package_by_id(
    Path(id): Path<PackageId>,
    Json(Package { metadata, data: _ }): Json<Package>,
) -> Result<(), StatusCode> {
    // if they put an id in the metadata, it should match the one they put in the path
    if metadata.id.as_ref() != "" && id != metadata.id {
        return Err(StatusCode::NOT_FOUND);
    }

    let db = database::get_database().await;

    let previous: PackageWithUrl = find_package_by_id(db, id, PACKAGE_FIELDS).await?;
    if previous.metadata.name != metadata.name || previous.metadata.version != metadata.version {
        return Err(StatusCode::NOT_FOUND);
    }

    // TODO: download / decode the uploaded package
    // TODO: score
    // TODO: upload to obj storage
    //       - remove old version?
    // TODO: update url in metadata to new location

    Ok(())
}

pub async fn post_package(
    Json(data): Json<PackageData>,
) -> Result<MyResponse<Package>, StatusCode> {
    // not yet implemeted:
    // 403: auth failed
    // 424: failed due to bad rating

    let RatedPackage {
        name,
        version,
        id,
        rating,
        content: _,
    } = scoring::rate_package(data).await.map_err(|e| {
        log::error!("{}", e);
        use RatingError::*;
        match e {
            MissingPackageJson | MissingRepository | UrlParseError(_) | CouldNotRate => {
                StatusCode::BAD_REQUEST
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    log::error!("{:?}", rating);
    return Err(StatusCode::IM_A_TEAPOT);
    todo!();
    // TODO: reject if score too low

    // TODO: upload to obj storage
    let url = "".into();

    let db = database::get_database().await;

    let metadata = PackageMetadata { name, version, id };

    let entry = DatabaseEntry {
        metadata,
        url,
        rating,
    };

    db.fluent()
        .insert()
        .into(database::METADATA)
        .document_id(&entry.metadata.id)
        .object(&entry)
        .execute()
        .await
        .map_err(|e| {
            log::error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 201: return package
    Ok(ok(entry.into()))
}

pub async fn get_rating_by_id(
    Path(id): Path<PackageId>,
) -> Result<MyResponse<PackageRating>, StatusCode> {
    let db = database::get_database().await;
    Ok(ok(find_package_by_id(db, id, RATING_FIELDS).await?))
}

/// Delete this version of the package.
// not in baseline requirements
pub async fn delete_package_by_id(Path(_id): Path<PackageId>) -> impl IntoResponse {
    // 200: package deleted
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED
}
