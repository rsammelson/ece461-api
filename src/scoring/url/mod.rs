#[cfg(test)]
mod tests;

use super::{RatingError::*, RatingResult};

use git_url_parse::GitUrl;
use once_cell::sync::OnceCell;
use reqwest::Client;
use semver::Version;
use serde::Deserialize;
use std::collections::HashMap;

pub fn get_client() -> Client {
    static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
    static CLIENT: OnceCell<Client> = OnceCell::new();
    CLIENT
        .get_or_init(|| {
            Client::builder()
                .user_agent(USER_AGENT)
                .https_only(true)
                .build()
                .unwrap()
        })
        .clone()
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct GithubUrl {
    pub(super) name: String,
    pub(super) owner: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct NpmUrl {
    pub(super) name: String,
}

impl<T> TryFrom<Result<GitUrl, T>> for GithubUrl {
    type Error = ();
    fn try_from(value: Result<GitUrl, T>) -> Result<Self, Self::Error> {
        match value {
            Ok(git_url) => git_url.try_into(),
            Err(_) => Err(()),
        }
    }
}

impl TryFrom<GitUrl> for GithubUrl {
    type Error = ();
    fn try_from(value: GitUrl) -> Result<Self, Self::Error> {
        match value {
            GitUrl {
                host: Some(host),
                name,
                owner: Some(owner),
                ..
            } if host.contains("github.com") => Ok(GithubUrl { name, owner }),
            _ => Err(()),
        }
    }
}

/// Transform something that could be put into the `"repository"` field of an npm `package.json`
/// into a `GithubUrl`.
pub(super) fn canonicalize_repo(url: &str) -> RatingResult<GithubUrl> {
    let err = || UrlParseError(url.to_string());

    if let Ok(git_url) = GitUrl::parse(url) {
        if let Ok(github_url) = git_url.try_into() {
            return Ok(github_url);
        }
    }

    // this is of the "shorthand" type, see
    // https://docs.npmjs.com/cli/v9/configuring-npm/package-json#repository
    // "abc/def" => github link
    // "github:abc/def" => github link
    // "blabla:abc/def" => not supported

    let len = url.split(':').count();
    let mut rest = url.split(':');
    if len > 1 && rest.next().unwrap() != "github" {
        return Err(err());
    }

    let mut split_slash = rest.next().ok_or_else(err)?.split('/');
    match (split_slash.next(), split_slash.next()) {
        (Some(owner), Some(name)) => Ok(GithubUrl {
            name: name.to_string(),
            owner: owner.to_string(),
        }),
        _ => Err(err()),
    }
}

// todo:
// - given npm url, query api
//      - from response, get link for tarball and repository link
//      - download, untar, zip, encode, save
//      - query github api
// - given github url
//      - query api
//      - download, encode, save
//          `https://api.github.com/repos/user/repo/zipball/`
//          -H "X-GitHub-Api-Version: 2022-11-28"
//          accept header application/vnd.github+json

#[derive(Debug, PartialEq, Eq)]
pub(super) enum UrlKind {
    Github(GithubUrl),
    Npm(NpmUrl),
}

impl TryFrom<&str> for UrlKind {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let url: ::url::Url = value
            .parse()
            .map_err(|e| log::error!("error parsing url: {}", e))?;

        let ::url::Host::Domain(host) = url.host().ok_or(())?
        else {
            return Err(())
        };

        // get parts of FQDN from right to left
        let mut domain_parts = host.rsplit('.');

        // check that the TLD is com
        match domain_parts.next() {
            Some(tld) if tld == "com" => (),
            _ => return Err(()),
        }

        match domain_parts.next() {
            Some(site) if site == "github" => {
                let mut split = url.path().trim_matches('/').split('/');
                if let (Some(owner), Some(name)) = (split.next(), split.next()) {
                    Ok(Self::Github(GithubUrl {
                        name: name.trim_end_matches(".git").to_owned(),
                        owner: owner.to_owned(),
                    }))
                } else {
                    Err(())
                }
            }
            Some(site) if site == "npmjs" => {
                let mut split = url.path().trim_matches('/').split('/');

                if let Some(mut name) = split.next() {
                    // might be of the form "npmjs.com/package/abc" or "npmjs.com/abc"
                    if name == "package" {
                        name = split.next().ok_or(())?;
                    }
                    Ok(Self::Npm(NpmUrl {
                        name: name.to_owned(),
                    }))
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub(super) struct NpmAbbrMetadata {
    #[serde(rename = "dist-tags")]
    pub(super) dist_tags: NpmDistTags,
    pub(super) versions: HashMap<Version, NpmVersion>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub(super) struct NpmDistTags {
    pub(super) latest: Version,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub(super) struct NpmVersion {
    pub(super) dist: NpmDist,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub(super) struct NpmDist {
    pub(super) tarball: String,
}
