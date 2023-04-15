mod id;
mod search;
pub use id::*;
pub use search::*;

use super::*;
use crate::user::AuthenticationRequest;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

/// Reset the registry
///
/// Reset the registry to a system default state.
// TODO: have to have this for baseline requirements
pub async fn reset_registry() -> impl IntoResponse {
    // 200: reset registry
    // 401: not authorized
    StatusCode::NOT_IMPLEMENTED
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
