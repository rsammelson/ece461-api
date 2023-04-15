use super::{RatingError::*, RatingResult};

use git_url_parse::GitUrl;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct GithubUrl {
    pub(super) name: String,
    pub(super) owner: String,
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

    if let Ok(git_url) = GitUrl::parse(&url) {
        if let Ok(github_url) = git_url.try_into() {
            return Ok(github_url);
        }
    }

    // this is of the "shorthand" type, see
    // [the npm docs](https://docs.npmjs.com/cli/v9/configuring-npm/package-json#repository)
    // "abc/def" => npm link
    // "github:abc/def" => github link
    // "blabla:abc/def" => not supported

    let len = url.split(':').count();
    let mut rest = url.split(':');
    if len > 1 {
        if rest.next().unwrap() != "github" {
            return Err(err());
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canon_repo_full_link() {
        let expected = GithubUrl {
            owner: "abc".to_string(),
            name: "def".to_string(),
        };
        let result = canonicalize_repo("https://github.com/abc/def").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn canon_repo_short() {
        let expected = GithubUrl {
            owner: "abc".to_string(),
            name: "def".to_string(),
        };
        let result = canonicalize_repo("abc/def").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn canon_repo_short_prefix() {
        let expected = GithubUrl {
            owner: "abc".to_string(),
            name: "def".to_string(),
        };
        let result = canonicalize_repo("github:abc/def").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn canon_repo_short_prefix_bad() {
        let result = canonicalize_repo("gitlab:abc/def").unwrap_err();
        assert!(matches!(result, UrlParseError(u) if u == "gitlab:abc/def"));
    }
}
