use firestore::FirestoreDb;

use super::*;
use crate::{database, package::*};

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
    Json(Package { metadata, data }): Json<Package>,
) -> Result<(), StatusCode> {
    if id != metadata.id {
        // ???
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

// TODO: this was updated to only include the data field
// needs to be pretty much entirely rewritten
pub async fn post_package(
    Json(Package { mut metadata, data }): Json<Package>,
) -> Result<MyResponse<Package>, StatusCode> {
    // not yet implemeted:
    // 403: auth failed
    // 424: failed due to bad rating
    let db = database::get_database().await;

    let prev_versions_count = db
        .fluent()
        .select()
        .from(database::METADATA)
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
        .into(database::METADATA)
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
