#[cfg(test)]
mod tests;

use super::{
    github,
    url::{canonicalize_repo, GithubUrl},
    version,
    RatingError::{self, *},
    RatingResult, ScoringData,
};
use crate::queries::types::PackageRating;

use git_url_parse::GitUrl;
use semver::Version;
use serde::Deserialize;
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    io::{self, Read, Seek, Write},
    path::Path,
};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Repository {
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PackageJson {
    Deep {
        name: String,
        version: Version,
        repository: Repository,
        dependencies: Option<HashMap<String, String>>,
    },
    Flat {
        name: String,
        version: Version,
        repository: String,
        dependencies: Option<HashMap<String, String>>,
    },
    #[allow(dead_code)]
    NoRepo {
        name: String,
        version: Version,
        dependencies: Option<HashMap<String, String>>,
    },
}

#[derive(Debug)]
struct PackageJsonVerified {
    name: String,
    version: Version,
    url: GithubUrl,
    dependencies: HashMap<String, String>,
}

impl TryFrom<PackageJson> for PackageJsonVerified {
    type Error = RatingError;
    fn try_from(value: PackageJson) -> Result<Self, Self::Error> {
        match value {
            PackageJson::NoRepo { .. } => Err(MissingRepository),
            PackageJson::Flat {
                name,
                version,
                repository,
                dependencies,
            } => Ok(PackageJsonVerified {
                name,
                version,
                url: canonicalize_repo(&repository)?,
                dependencies: match dependencies {
                    Some(map) => map,
                    None => HashMap::new(),
                },
            }),
            PackageJson::Deep {
                name,
                version,
                repository: Repository { url },
                dependencies,
            } => Ok(PackageJsonVerified {
                name,
                version,
                url: GitUrl::parse(&url)
                    .try_into()
                    .map_err(|_| UrlParseError(url))?,
                dependencies: match dependencies {
                    Some(map) => map,
                    None => HashMap::new(),
                },
            }),
        }
    }
}

pub(super) async fn rating_from_path<P: AsRef<Path>>(
    path: P,
) -> RatingResult<(String, Version, PackageRating)> {
    let readme_exists = find_file(&path, |name| {
        name.eq_ignore_ascii_case("readme") || name.eq_ignore_ascii_case("readme.md")
    })
    .is_some();

    let PackageJsonVerified {
        name,
        version,
        url,
        dependencies,
    } = serde_json::from_reader::<_, PackageJson>(io::BufReader::new(fs::File::open(
        find_file(&path, |name| name == "package.json")
            .ok_or_else(|| MissingPackageJson)?
            .into_path(),
    )?))?
    .try_into()?;

    let scoring_data = ScoringData {
        readme_exists,
        ..github::graphql::query(url).await?
    };

    let good_pinning_practice = version::score_versionreq_pinned(dependencies);
    let pull_request = 0.;

    let scores = (scoring_data, good_pinning_practice, pull_request).into();

    Ok((name, version, scores))
}

fn find_file<P: AsRef<Path>, F: Fn(&OsStr) -> bool>(haystack: P, needle: F) -> Option<DirEntry> {
    let walker = WalkDir::new(haystack).into_iter();
    walker
        .filter_map(|e| e.ok())
        .find(|e| needle(e.file_name()))
}

/// https://github.com/zip-rs/zip/blob/master/examples/write\_dir.rs
fn zip_dir_internal<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = zip::write::FileOptions::default();

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            zip.add_directory(name.to_string_lossy(), options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}

pub fn zip_dir<T>(src_dir: &str, dst: T) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    if !Path::new(src_dir).is_dir() {
        return Err(zip::result::ZipError::FileNotFound);
    }

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    zip_dir_internal(&mut it.filter_map(|e| e.ok()), src_dir, dst)?;

    Ok(())
}
