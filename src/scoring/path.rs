use super::{
    github,
    url::{canonicalize_repo, GithubUrl},
    RatingError::{self, *},
    RatingResult, ScoringData,
};
use crate::queries::types::PackageRating;

use git_url_parse::GitUrl;
use semver::Version;
use serde::Deserialize;
use std::{
    collections::VecDeque,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
struct Repository {
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PackageJson {
    DeepRepo {
        name: String,
        version: Version,
        repository: Repository,
    },
    FlatRepo {
        name: String,
        version: Version,
        repository: String,
    },
    #[allow(dead_code)]
    NoRepo { name: String, version: Version },
}

#[derive(Debug)]
struct PackageJsonVerified {
    name: String,
    version: Version,
    url: GithubUrl,
}

impl TryFrom<PackageJson> for PackageJsonVerified {
    type Error = RatingError;
    fn try_from(value: PackageJson) -> Result<Self, Self::Error> {
        match value {
            PackageJson::NoRepo { .. } => Err(MissingRepository),
            PackageJson::FlatRepo {
                name,
                version,
                repository,
            } => Ok(PackageJsonVerified {
                name,
                version,
                url: canonicalize_repo(&repository)?,
            }),
            PackageJson::DeepRepo {
                name,
                version,
                repository: Repository { url },
            } => Ok(PackageJsonVerified {
                name,
                version,
                url: GitUrl::parse(&url)
                    .try_into()
                    .map_err(|_| UrlParseError(url))?,
            }),
        }
    }
}

pub(super) async fn rating_from_path<P: AsRef<Path>>(
    path: P,
) -> RatingResult<(String, Version, PackageRating)> {
    let readme_exists = find_file(&path, |name| name.to_lowercase().contains("readme"))?.is_some();

    let PackageJsonVerified { name, version, url } =
        serde_json::from_reader::<_, PackageJson>(io::BufReader::new(fs::File::open(
            find_file(&path, |name| name == "package.json")?.ok_or_else(|| MissingPackageJson)?,
        )?))?
        .try_into()?;

    let data = ScoringData {
        readme_exists,
        ..github::graphql::query(url).await?
    };

    let scores = data.into();

    Ok((name, version, scores))
}

/// Breadth first traversal of the filesystem starting at the haystack, searching for a file whose
/// name satisfies the given closure
fn find_file<P: AsRef<Path>, F: Fn(String) -> bool>(
    haystack: P,
    needle: F,
) -> Result<Option<PathBuf>, io::Error> {
    let mut barn = VecDeque::from([haystack.as_ref().to_path_buf()]);
    while let Some(haystack) = barn.pop_front() {
        for entry in fs::read_dir(haystack)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                barn.push_back(path);
            } else {
                let name = match entry.file_name().into_string() {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                if needle(name) {
                    return Ok(Some(path));
                }
            }
        }
    }
    Ok(None)
}
