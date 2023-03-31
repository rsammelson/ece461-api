use crate::package::*;
use crate::user::*;
use crate::{DB, METADATA};

use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use firestore::*;
use futures::{future, stream::BoxStream, StreamExt};

/// Get the packages from the registry.
///
/// Get any packages fitting the query. Search for packages satisfying the indicated query.
/// If you want to enumerate all packages, provide an array with a single PackageQuery whose name is "*".
/// The response is paginated; the response header includes the offset to use in the next query.
#[post("/packages")]
pub async fn search_packages(search: web::Json<Vec<SearchQuery>>) -> impl Responder {
    // 200: list of packages
    // 413: too many packages returned (shouldn't happen? it's paginated)
    let search = search.into_inner();
    HttpResponse::NotImplemented().json(
        search
            .iter()
            .map(|_| PackageMetadata::default())
            .collect::<Vec<_>>(),
    )
}

/// Reset the registry
///
/// Reset the registry to a system default state.
#[delete("/reset")]
pub async fn reset_registry() -> impl Responder {
    // 200: reset registry
    // 401: not authorized
    HttpResponse::NotImplemented().finish()
}

/// Interact with the package with this ID
///
/// Return this package.
#[get("/package/{id}")]
pub async fn get_package_by_id(_id: web::Path<PackageId>) -> impl Responder {
    // 200: return package
    // 404: does not exist
    HttpResponse::NotImplemented().json(Package::default())
}

/// Update the content of the package.
///
/// The name, version, and ID must match.
/// The package contents (from PackageData) will replace the previous contents.
#[put("/package/{id}")]
pub async fn update_package_by_id(
    _id: web::Path<PackageId>,
    _info: web::Json<Package>,
) -> impl Responder {
    // 200: package updated
    // 404: does not exist
    HttpResponse::NotImplemented().finish()
}

/// Delete this version of the package.
#[delete("/package/{id}")]
pub async fn delete_package_by_id(_id: web::Path<PackageId>) -> impl Responder {
    // 200: package deleted
    // 404: does not exist
    HttpResponse::NotImplemented().finish()
}

#[post("/package")]
pub async fn post_package(to_upload: web::Json<Package>) -> impl Responder {
    // not yet implemeted:
    // 403: auth failed
    // 424: failed due to bad rating
    let to_upload = to_upload.into_inner();

    let prev_versions: FirestoreResult<BoxStream<PackageMetadata>> = DB
        .fluent()
        .select()
        .from(METADATA)
        .filter(|q| {
            q.field(path!(PackageMetadata::name))
                .eq(&to_upload.metadata.name)
        })
        .obj()
        .stream_query()
        .await;

    let prev_versions = match prev_versions {
        Ok(p) => p,
        // 500: ??? some database error
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if prev_versions
        .filter(|m| future::ready(m.version == to_upload.metadata.version))
        .count()
        .await
        >= 1
    {
        // 409: package already exists
        return HttpResponse::Conflict().finish();
    }

    let id = PackageId::new();
    let response: FirestoreResult<PackageMetadata> = DB
        .fluent()
        .insert()
        .into(METADATA)
        .document_id(id.to_string())
        .object(&to_upload.metadata)
        .execute()
        .await;

    match response {
        // 201: return package, with correct ID
        Ok(PackageMetadata { id, .. }) => HttpResponse::Created().json(Package {
            metadata: PackageMetadata {
                id,
                ..to_upload.metadata
            },
            ..to_upload
        }),
        // 500: ??? some database error
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/package/{id}/rate")]
pub async fn get_rating_by_id(_id: web::Path<PackageId>) -> impl Responder {
    // 200: return rating iff all rated
    // 404: does not exist
    // 500: package rating error
    HttpResponse::NotImplemented().json(PackageRating::default())
}

/// Create an access token.
#[put("/authenticate")]
pub async fn authenticate(auth: web::Json<AuthenticationRequest>) -> impl Responder {
    // 200: return token
    // 401: invalid user/password
    // 501: not implemented
    let auth = auth.into_inner();
    HttpResponse::NotImplemented().json(AuthenticationToken::new(auth))
}

/// Return the history of this package (all versions).
#[get("/package/byName/{name}")]
pub async fn get_package_by_name(name: web::Path<String>) -> impl Responder {
    // 200: return package history
    // 404: does not exist
    let name = name.into_inner();
    HttpResponse::NotImplemented().json(vec![PackageHistoryEntry {
        metadata: PackageMetadata {
            name,
            ..Default::default()
        },
        ..Default::default()
    }])
}

/// Delete all versions of this package.
#[delete("/package/byName/{name}")]
pub async fn delete_package_by_name(_name: web::Path<String>) -> impl Responder {
    // 200: package deleted
    // 404: does not exist
    HttpResponse::NotImplemented().finish()
}

/// Get any packages fitting the regular expression.
///
/// Search for a package using regular expression over package names and READMEs.
#[get("/package/byRegEx")]
pub async fn get_package_by_regex(_regex: web::Json<String>) -> impl Responder {
    // 200: return list of packages
    // 404: no packages found
    HttpResponse::NotImplemented().json(vec![PackageMetadata::default()])
}
