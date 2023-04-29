use firestore::FirestoreDb;

use super::{ok, types::*, MyResponse};
use crate::{
    database::{self, DatabaseEntry},
    scoring::{self, RatedPackage, RatingError},
    storage::CloudStorage,
};

use axum::{
    extract::{Json, Path},
    http::StatusCode,
};

const MIN_ALLOWED_NET_SCORE: f64 = 0.5;

async fn find_package_by_id<F, T>(
    db: &FirestoreDb,
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

    query_result.into_iter().next().ok_or(StatusCode::NOT_FOUND)
}

/// Some of the errors returned by scoring are server errors, some are because of a bad request
/// This takes the rating error and transforms it into the appropriate status code (400 or 500)
fn scoring_err_to_response(e: RatingError) -> StatusCode {
    log::error!("scoring error: {:?}", e);
    use RatingError::*;
    match e {
        MissingPackageJson | MissingRepository | UrlParseError(_) | CouldNotRate => {
            StatusCode::BAD_REQUEST
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR,
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
    find_package_by_id(&db, id, PACKAGE_FIELDS).await.map(ok)
}

/// Update the content of the package.
///
/// The name, version, and ID must match.
/// The package contents (from PackageData) will replace the previous contents.
pub async fn update_package_by_id(
    Path(path_id): Path<PackageId>,
    Json(Package { metadata, data }): Json<Package>,
) -> Result<(), StatusCode> {
    // if they put an id in the metadata, it should match the one they put in the path
    if metadata.id.as_ref() != "" && path_id != metadata.id {
        return Err(StatusCode::NOT_FOUND);
    }

    let db = database::get_database().await;

    let previous: PackageWithUrl = find_package_by_id(&db, path_id, PACKAGE_FIELDS).await?;
    if previous.metadata.name != metadata.name || previous.metadata.version != metadata.version {
        return Err(StatusCode::NOT_FOUND);
    }

    let RatedPackage {
        name,
        version,
        rating,
        content,
        ..
    } = scoring::rate_package(data)
        .await
        .map_err(scoring_err_to_response)?;

    if name != metadata.name || version != metadata.version {
        // trying to upload wrong package
        log::error!(
            concat!(
                "Package update: metadata from package contents does not match stored metadata\n",
                "Extracted name: {}\n",
                "Stored name: {}\n",
                "Extracted version: {}\n",
                "Stored version: {}",
            ),
            name,
            previous.metadata.name,
            version,
            previous.metadata.version
        );
        return Err(StatusCode::NOT_FOUND);
    }

    if rating.net_score < MIN_ALLOWED_NET_SCORE {
        return Err(StatusCode::FAILED_DEPENDENCY);
    }

    // upload to obj storage
    let storage = CloudStorage::new()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let url = storage
        .put_object(name, content)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let entry = DatabaseEntry {
        metadata,
        url,
        rating,
    };

    db.fluent()
        .update()
        .fields(RATING_FIELDS.iter().chain([&database::URL]))
        .in_col(database::METADATA)
        .document_id(previous.metadata.id)
        .object(&entry)
        .execute()
        .await
        .map_err(|e| {
            log::error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(())
}

pub async fn post_package(
    Json(data): Json<PackageData>,
) -> Result<MyResponse<Package>, StatusCode> {
    let db = database::get_database().await;

    let RatedPackage {
        name,
        version,
        id,
        rating,
        content,
    } = scoring::rate_package(data)
        .await
        .map_err(scoring_err_to_response)?;

    if rating.net_score < MIN_ALLOWED_NET_SCORE {
        return Err(StatusCode::FAILED_DEPENDENCY);
    }

    let query = db
        .fluent()
        .select()
        .fields(PACKAGE_FIELDS)
        .from(database::METADATA)
        .limit(1)
        .filter(|q| q.field(database::NAME).eq(&name));

    let query_result = query.query().await.map_err(|e| {
        log::error!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if query_result.len() >= 1 {
        return Err(StatusCode::CONFLICT);
    }

    // upload to obj storage
    let storage = CloudStorage::new().await.map_err(|e| {
        log::error!("cloud storage handle error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let url = storage
        .put_object(name.clone(), content)
        .await
        .map_err(|e| {
            log::error!("cloud storage put error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

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
    Ok(ok(find_package_by_id(&db, id, RATING_FIELDS).await?))
}

/// Delete this version of the package.
// not in baseline requirements
pub async fn delete_package_by_id(Path(_id): Path<PackageId>) -> StatusCode {
    // 200: package deleted
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED
}
