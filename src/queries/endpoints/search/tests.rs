use super::*;

use test_log::test;

use axum::http::header;
use semver::Version;
use uuid::Uuid;

#[test]
fn des_search_version_single() {
    let data = r#"{"Name":"to_search","Version":"1.0"}"#;

    let deserialized: SearchQuery = serde_json::from_str(data).unwrap();
    assert_eq!(
        deserialized,
        SearchQuery {
            name: "to_search".to_string(),
            version: Some(VersionReq::parse("1.0").unwrap())
        }
    );
}

#[test]
fn des_search_version_range() {
    let data = r#"{"Name":"to_search","Version":">=1.2.3, <1.8.0"}"#;

    let deserialized: SearchQuery = serde_json::from_str(data).unwrap();
    assert_eq!(
        deserialized,
        SearchQuery {
            name: "to_search".to_string(),
            version: Some(VersionReq::parse(">=1.2.3,<1.8.0").unwrap())
        }
    );
}

#[test]
fn des_search_no_version() {
    let data = r#"{"Name":"to_search"}"#;

    let deserialized: SearchQuery = serde_json::from_str(data).unwrap();
    assert_eq!(
        deserialized,
        SearchQuery {
            name: "to_search".to_string(),
            version: None,
        }
    );
}

#[test]
fn des_search_name_required() {
    let data = r#"{"Version":"0.0.0"}"#;

    let deserialized: Result<SearchQuery, _> = serde_json::from_str(data);
    if let Ok(_) = deserialized {
        panic!("Expected \"Name\" to be required for a `SearchQuery`");
    }
}

#[test(tokio::test)]
#[ignore]
async fn query_search() {
    let query = vec![SearchQuery {
        name: "to_search".to_string(),
        version: None,
    }];
    let MyResponse {
        code,
        headers,
        body,
    } = search_packages(Query(Offset { offset: None }), Json(query))
        .await
        .unwrap();

    assert_eq!(code, StatusCode::OK);
    assert_eq!(
        headers,
        vec![
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json")
            ),
            (
                HeaderName::from_static("offset"),
                HeaderValue::from_static(concat!(
                    "1.0.1",
                    ",",
                    "67e55044-10b1-426f-9247-bb680e5fe0c8"
                ))
            )
        ]
    );

    assert_eq!(
        body,
        vec![
            PackageMetadata {
                name: "to_search".to_string(),
                version: Version::parse("1.0.0").unwrap(),
                id: Uuid::nil().into()
            },
            PackageMetadata {
                name: "to_search".to_string(),
                version: Version::parse("1.0.1").unwrap(),
                id: "67e55044-10b1-426f-9247-bb680e5fe0c8".into()
            },
        ]
    )
}

#[test(tokio::test)]
#[ignore]
async fn query_search_offset() {
    let query = vec![SearchQuery {
        name: "to_search".to_string(),
        version: None,
    }];
    let MyResponse {
        code,
        headers,
        body,
    } = search_packages(
        Query(Offset {
            offset: Some(
                concat!("1.0.1", ",", "67e55044-10b1-426f-9247-bb680e5fe0c8",).to_string(),
            ),
        }),
        Json(query),
    )
    .await
    .unwrap();

    assert_eq!(code, StatusCode::OK);
    assert_eq!(
        headers,
        vec![
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json")
            ),
            (
                HeaderName::from_static("offset"),
                HeaderValue::from_static(concat!(
                    "2.0.0",
                    ",",
                    "38e5f63a-4a59-4187-a0e7-3364b2c530c3"
                ))
            )
        ]
    );

    assert_eq!(
        body,
        vec![
            PackageMetadata {
                name: "to_search".to_string(),
                version: Version::parse("1.0.3").unwrap(),
                id: "04b459e2-a696-4531-9e7a-ae931ed38bc4".into()
            },
            PackageMetadata {
                name: "to_search".to_string(),
                version: Version::parse("2.0.0").unwrap(),
                id: "38e5f63a-4a59-4187-a0e7-3364b2c530c3".into()
            },
        ]
    )
}

#[test(tokio::test)]
#[ignore]
async fn query_search_version_simple_all() {
    let query = vec![SearchQuery {
        name: "to_search".to_string(),
        version: Some(VersionReq::parse("2.0.0").unwrap()),
    }];
    let MyResponse {
        code,
        headers,
        body,
    } = search_packages(Query(Offset { offset: None }), Json(query))
        .await
        .unwrap();

    assert_eq!(code, StatusCode::OK);
    assert_eq!(
        headers,
        vec![
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json")
            ),
            (
                HeaderName::from_static("offset"),
                HeaderValue::from_static(concat!(
                    "2.1.3",
                    ",",
                    "436b4766-c31c-47ca-84fd-8ed522c49191"
                ))
            )
        ]
    );

    assert_eq!(
        body,
        vec![
            PackageMetadata {
                name: "to_search".to_string(),
                version: Version::parse("2.0.0").unwrap(),
                id: "38e5f63a-4a59-4187-a0e7-3364b2c530c3".into()
            },
            PackageMetadata {
                name: "to_search".to_string(),
                version: Version::parse("2.1.3").unwrap(),
                id: "436b4766-c31c-47ca-84fd-8ed522c49191".into()
            },
        ]
    )
}

#[test(tokio::test)]
#[ignore]
async fn query_search_version_simple_equal() {
    let query = vec![SearchQuery {
        name: "to_search".to_string(),
        version: Some(VersionReq::parse("=1.0.0").unwrap()),
    }];
    let MyResponse {
        code,
        headers,
        body,
    } = search_packages(Query(Offset { offset: None }), Json(query))
        .await
        .unwrap();

    // should only match one thing, so no offset header
    assert_eq!(code, StatusCode::OK);
    assert_eq!(
        headers,
        vec![(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json")
        )]
    );

    assert_eq!(
        body,
        vec![PackageMetadata {
            name: "to_search".to_string(),
            version: Version::parse("1.0.0").unwrap(),
            id: Uuid::nil().into()
        },]
    )
}

#[test(tokio::test)]
#[ignore]
async fn query_search_version_range() {
    let query = vec![SearchQuery {
        name: "to_search".to_string(),
        version: Some(VersionReq::parse(">=1.0.1,<1.1").unwrap()),
    }];
    let MyResponse {
        code,
        headers,
        body,
    } = search_packages(Query(Offset { offset: None }), Json(query))
        .await
        .unwrap();

    assert_eq!(code, StatusCode::OK);
    assert_eq!(
        headers,
        vec![
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json")
            ),
            (
                HeaderName::from_static("offset"),
                HeaderValue::from_static(concat!(
                    "1.0.3",
                    ",",
                    "04b459e2-a696-4531-9e7a-ae931ed38bc4"
                ))
            )
        ]
    );

    assert_eq!(
        body,
        vec![
            PackageMetadata {
                name: "to_search".to_string(),
                version: Version::parse("1.0.1").unwrap(),
                id: "67e55044-10b1-426f-9247-bb680e5fe0c8".into()
            },
            PackageMetadata {
                name: "to_search".to_string(),
                version: Version::parse("1.0.3").unwrap(),
                id: "04b459e2-a696-4531-9e7a-ae931ed38bc4".into()
            },
        ]
    )
}

#[test(tokio::test)]
#[ignore]
async fn query_search_all_version() {
    let query = vec![SearchQuery {
        name: "*".to_string(),
        version: Some(VersionReq::parse("=1.0.1").unwrap()),
    }];
    let MyResponse {
        code,
        headers,
        body,
    } = search_packages(Query(Offset { offset: None }), Json(query))
        .await
        .unwrap();

    assert_eq!(code, StatusCode::OK);
    assert_eq!(
        headers,
        vec![
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json")
            ),
            (
                HeaderName::from_static("offset"),
                HeaderValue::from_static(concat!(
                    "1.0.1",
                    ",",
                    "e853d161-5163-4bfe-a535-f131a4a357d1"
                ))
            )
        ]
    );

    assert_eq!(
        body,
        vec![
            PackageMetadata {
                name: "to_search".to_string(),
                version: Version::parse("1.0.1").unwrap(),
                id: "67e55044-10b1-426f-9247-bb680e5fe0c8".into()
            },
            PackageMetadata {
                name: "to_not_search".to_string(),
                version: Version::parse("1.0.1").unwrap(),
                id: "e853d161-5163-4bfe-a535-f131a4a357d1".into()
            },
        ]
    )
}

#[test(tokio::test)]
#[ignore]
async fn query_search_all_version_no_result() {
    let query = vec![SearchQuery {
        name: "*".to_string(),
        version: Some(VersionReq::parse("=1.0.1").unwrap()),
    }];
    let MyResponse {
        code,
        headers,
        body,
    } = search_packages(
        Query(Offset {
            offset: Some(concat!("1.0.1", ",", "e853d161-5163-4bfe-a535-f131a4a357d1").to_string()),
        }),
        Json(query),
    )
    .await
    .unwrap();

    // shouldn't have anything in response, so no header
    assert_eq!(code, StatusCode::OK);
    assert_eq!(
        headers,
        vec![(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json")
        ),]
    );

    assert_eq!(body, vec![])
}

#[test]
fn offset_parse() {
    assert!(Offset::parse(concat!(
        "1.2.3",
        ",",
        "04b459e2-a696-4531-9e7a-ae931ed38bc4"
    ))
    .is_some())
}
