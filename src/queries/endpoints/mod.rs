mod id;
mod search;
use firestore::{FirestoreStreamingBatchWriteOptions, FirestoreStreamingBatchWriter};
pub use id::*;
pub use search::*;
use serde::Deserialize;
use tokio::join;

use super::{types::PackageId, *};
use crate::{database, storage::CloudStorage, user::AuthenticationRequest};

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

async fn clear_metadata() -> Result<(), StatusCode> {
    #[derive(Deserialize, Debug)]
    struct JustId {
        #[serde(rename = "ID")]
        id: PackageId,
    }

    // read all ids
    let db = database::get_database().await;
    let all_ids: Vec<JustId> = db
        .fluent()
        .select()
        .fields([database::ID])
        .from(database::METADATA)
        .obj()
        .query()
        .await
        .map_err(|e| {
            log::error!("error reading IDs during delete: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // create a batch writer
    let (batch_stream, _) =
        FirestoreStreamingBatchWriter::new(db.clone(), FirestoreStreamingBatchWriteOptions::new())
            .await
            .map_err(|e| {
                log::error!("something went wrong creating a batch: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    // add all delete operations to batch writer
    let mut batch = batch_stream.new_batch();
    for id in all_ids {
        batch
            .delete_by_id(database::METADATA, id.id, None)
            .map_err(|e| {
                log::error!("batch.delete_by_id() returned err: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    // execute batch write
    batch.write().await.map_err(|e| {
        log::error!("while executing metadata deletions: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    // not sure if this is needed?
    batch_stream.finish().await;

    Ok(())
}

async fn clear_bucket() -> Result<(), StatusCode> {
    let storage = CloudStorage::new().await.map_err(|e| {
        log::error!("while getting storage bucket: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    storage.delete_all().await.map_err(|_| {
        log::error!("while executing bucket deletion");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(())
}

/// Reset the registry
///
/// Reset the registry to a system default state.
// TODO: clear metadata
pub async fn reset_registry() -> Result<StatusCode, StatusCode> {
    // 200: reset registry
    match join!(clear_metadata(), clear_bucket()) {
        (Err(_), _) | (_, Err(_)) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        _ => Ok(StatusCode::OK),
    }
}

/// Create an access token.
// not in baseline requirements
pub async fn authenticate(Json(_auth): Json<AuthenticationRequest>) -> impl IntoResponse {
    // 200: return token
    // 401: invalid user/password
    // 501: not implemented
    StatusCode::NOT_IMPLEMENTED
}

/// Return the history of this package (all versions).
// not in baseline requirements
pub async fn get_package_by_name(Path(_name): Path<String>) -> impl IntoResponse {
    // 200: return package history
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED
}

/// Delete all versions of this package.
// not in baseline requirements
pub async fn delete_package_by_name(Path(_name): Path<String>) -> impl IntoResponse {
    // 200: package deleted
    // 404: does not exist
    StatusCode::NOT_IMPLEMENTED
}

/// Get any packages fitting the regular expression.
///
/// Search for a package using regular expression over package names and READMEs.
// not in baseline requirements
pub async fn get_package_by_regex(_regex: String) -> impl IntoResponse {
    // 200: return list of packages
    // 404: no packages found
    StatusCode::NOT_IMPLEMENTED
}
