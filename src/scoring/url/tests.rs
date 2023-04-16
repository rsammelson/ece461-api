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

#[test]
fn npm_metadata_deserialize() {
    let data = r#"{"dist-tags":{"latest":"1.0.0"},"modified":"2015-05-16T22:27:54.741Z","name":"tiny-tarball","versions":{"1.0.0":{"_hasShrinkwrap":false,"directories":{},"dist":{"shasum":"bbf102d5ae73afe2c553295e0fb02230216f65b1","tarball":"https://registry.npmjs.org/tiny-tarball/-/tiny-tarball-1.0.0.tgz"},"name":"tiny-tarball","version":"1.0.0"}}}"#;
    let expected = NpmAbbrMetadata {
        dist_tags: NpmDistTags {
            latest: "1.0.0".parse().unwrap(),
        },
        versions: HashMap::from([(
            "1.0.0".parse().unwrap(),
            NpmVersion {
                dist: NpmDist {
                    tarball: "https://registry.npmjs.org/tiny-tarball/-/tiny-tarball-1.0.0.tgz"
                        .to_string(),
                },
            },
        )]),
    };

    let des: NpmAbbrMetadata = serde_json::from_str(data).unwrap();
    assert_eq!(des, expected);
}

#[test]
fn url_kind_parse() {
    let data = "https://npmjs.com/abc";
    let result: UrlKind = data.try_into().unwrap();
    let expected = UrlKind::Npm(NpmUrl {
        name: "abc".to_owned(),
    });
    assert_eq!(result, expected);

    let data = "https://npmjs.com/abc/def";
    let result: UrlKind = data.try_into().unwrap();
    let expected = UrlKind::Npm(NpmUrl {
        name: "abc".to_owned(),
    });
    assert_eq!(result, expected);

    let data = "https://npmjs.com/package/abc/def";
    let result: UrlKind = data.try_into().unwrap();
    let expected = UrlKind::Npm(NpmUrl {
        name: "abc".to_owned(),
    });
    assert_eq!(result, expected);

    let data = "https://github.com/abc/def";
    let result: UrlKind = data.try_into().unwrap();
    let expected = UrlKind::Github(GithubUrl {
        name: "def".to_owned(),
        owner: "abc".to_owned(),
    });
    assert_eq!(result, expected);

    let data = "https://badsite.com/abc/def";
    <&str as TryInto<UrlKind>>::try_into(data).unwrap_err();
}
