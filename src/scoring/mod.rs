mod github;
mod path;
mod url;
mod version;

use self::url::{get_client, NpmAbbrMetadata, NpmDist, NpmDistTags, NpmVersion, UrlKind};
use crate::queries::types::{PackageData, PackageId, PackageRating};

use base64::{engine::general_purpose, read::DecoderReader};
use libflate::gzip;
use semver::Version;
use std::io::{self, Read};
use zip::ZipArchive;

#[derive(thiserror::Error, Debug)]
pub enum RatingError {
    #[error("could not rate package")]
    CouldNotRate,
    #[error("did not find a package.json")]
    MissingPackageJson,
    #[error("package.json did not contain a repository link")]
    MissingRepository,
    #[error("npm api response did not contain tarball for latest version")]
    CouldNotGetLatestVersion,
    #[error("could not convert repository url: `{0}`")]
    UrlParseError(String),
    #[error("{0}")]
    GraphQlError(#[from] github::GraphQlError),
    #[error("{0}")]
    ZipError(#[from] zip::result::ZipError),
    #[error("{0}")]
    IoError(#[from] io::Error),
    #[error("base64 error: `{0}`")]
    Base64Error(io::Error),
    #[error("{0}")]
    DeserializeError(#[from] serde_json::Error),
    #[error("{0}")]
    ReqwestError(#[from] reqwest::Error),
}
use RatingError::*;

type RatingResult<T> = Result<T, RatingError>;

pub struct RatedPackage {
    pub name: String,
    pub version: Version,
    pub id: PackageId,
    pub rating: PackageRating,
    pub content: Vec<u8>,
}

pub async fn rate_package(package: PackageData) -> RatingResult<RatedPackage> {
    match package {
        PackageData::Content(content) => Ok(from_content(content.into_bytes()).await?),
        PackageData::Url(url) => from_url(&url).await,
        _ => Err(CouldNotRate),
    }
}

async fn from_content(content: Vec<u8>) -> RatingResult<RatedPackage> {
    let mut buf = Vec::new();
    DecoderReader::new(content.as_slice(), &general_purpose::STANDARD)
        .read_to_end(&mut buf)
        .map_err(Base64Error)?;
    let buf = io::Cursor::new(buf);

    let id = PackageId::new();
    let path = format!("/tmp/{}", id.as_ref());

    let result = from_content_internal(buf, &path).await;

    let _ = std::fs::remove_dir_all(&path)
        .map_err(|e| log::error!("Error removing files after scoring: `{}`", e));

    let (name, version, rating) = result?;
    Ok(RatedPackage {
        name,
        version,
        id,
        rating,
        content,
    })
}

// to catch errors and still remove temporary files if so
async fn from_content_internal(
    buf: io::Cursor<Vec<u8>>,
    path: &str,
) -> RatingResult<(String, Version, PackageRating)> {
    ZipArchive::new(buf)?.extract(path)?;
    path::rating_from_path(path).await
}

async fn from_url(url: &str) -> RatingResult<RatedPackage> {
    let id = PackageId::new();
    let path = format!("/tmp/{}", id.as_ref());

    let result = from_url_internal(url, &path, id).await;

    let _ = std::fs::remove_dir_all(&path)
        .map_err(|e| log::error!("Error removing files after scoring: `{}`", e));

    result
}

// TODO: content should be base64 encoded before returning
// to catch errors and still remove temporary files if so
async fn from_url_internal(url: &str, path: &str, id: PackageId) -> RatingResult<RatedPackage> {
    let url = url.try_into().map_err(|_| UrlParseError(url.to_string()))?;

    let content = match url {
        UrlKind::Github(url) => {
            let content = get_client()
                .get(format!(
                    "https://api.github.com/repos/{}/{}/zipball",
                    url.owner, url.name
                ))
                .header("X-GitHub-Api-Version", "2022-11-28")
                .send()
                .await?
                .bytes()
                .await?;

            ZipArchive::new(io::Cursor::new(&content[..]))
                .unwrap()
                .extract(path)
                .unwrap();
            content.into()
        }
        UrlKind::Npm(url) => {
            let client = get_client();
            let NpmAbbrMetadata {
                dist_tags: NpmDistTags { latest },
                versions,
            } = client
                .get(format!("https://registry.npmjs.org/{}", url.name))
                .header("Accept", "application/vnd.npm.install-v1+json")
                .send()
                .await?
                .json()
                .await?;

            let NpmVersion {
                dist: NpmDist { tarball },
            } = versions
                .get(&latest)
                .ok_or_else(|| CouldNotGetLatestVersion)?;

            let tar_gz = client.get(tarball).send().await?.bytes().await?;
            tar::Archive::new(gzip::Decoder::new(&tar_gz[..])?).unpack(path)?;

            let mut content = Vec::new();
            path::zip_dir(path, io::Cursor::new(&mut content))?;
            content
        }
    };

    let (name, version, rating) = path::rating_from_path(&path).await?;
    Ok(RatedPackage {
        name,
        version,
        id,
        rating,
        content,
    })
}

struct ScoringData {
    readme_exists: bool,
    documentation_exists: bool,
    issues_closed: usize,
    issues_total: usize,
    num_contributors: usize,
    weeks_since_last_issue: f64,
    license_correct: bool,
}

impl From<(ScoringData, f64, f64)> for PackageRating {
    fn from(
        (
            ScoringData {
                readme_exists,
                documentation_exists,
                issues_closed,
                issues_total,
                num_contributors,
                weeks_since_last_issue,
                license_correct,
            },
            good_pinning_practice,
            pull_request,
        ): (ScoringData, f64, f64),
    ) -> Self {
        let bus_factor = 1. - (1. / num_contributors.max(1) as f64);
        let correctness = (issues_closed as f64 / issues_total as f64).max(0.).min(1.);
        let ramp_up =
            if readme_exists { 0.5 } else { 0. } + if documentation_exists { 0.5 } else { 0. };
        let responsive_maintainer = (1. / weeks_since_last_issue).max(0.).min(1.);
        let license_score = if license_correct { 1. } else { 0. };

        PackageRating {
            bus_factor,
            correctness,
            ramp_up,
            responsive_maintainer,
            license_score,
            good_pinning_practice,
            pull_request,
            net_score: 0.,
        }
        .set_net_score()
    }
}
