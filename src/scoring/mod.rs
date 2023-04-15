mod github;
mod path;
mod url;

use crate::queries::types::{PackageData, PackageId, PackageRating};

use base64::{engine::general_purpose, read::DecoderReader};
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
        .map_err(|e| Base64Error(e))?;
    let buf = io::Cursor::new(buf);

    let id = PackageId::new();
    let path = format!("/tmp/{}", id.as_ref());

    ZipArchive::new(buf)?.extract(&path)?;

    let (name, version, rating) = path::rating_from_path(&path).await?;

    // can still score if the files weren't removed
    let _ = std::fs::remove_dir_all(&path)
        .map_err(|e| log::error!("Error removing files after scoring: `{}`", e));

    Ok(RatedPackage {
        name,
        version,
        id,
        rating,
        content,
    })
}

async fn from_url(_url: &str) -> RatingResult<RatedPackage> {
    todo!()
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

impl From<ScoringData> for PackageRating {
    fn from(
        ScoringData {
            readme_exists,
            documentation_exists,
            issues_closed,
            issues_total,
            num_contributors,
            weeks_since_last_issue,
            license_correct,
        }: ScoringData,
    ) -> Self {
        let bus_factor = 1. - (1. / num_contributors.max(1) as f64);
        let correctness = (issues_closed as f64 / issues_total as f64).max(0.).min(1.);
        let ramp_up =
            if readme_exists { 0.5 } else { 0. } + if documentation_exists { 0.5 } else { 0. };
        let responsive_maintainer = (1. / weeks_since_last_issue).max(0.).min(1.);
        let license_score = if license_correct { 1. } else { 0. };
        let good_pinning_practice = 0.;
        PackageRating {
            bus_factor,
            correctness,
            ramp_up,
            responsive_maintainer,
            license_score,
            good_pinning_practice,
        }
    }
}
