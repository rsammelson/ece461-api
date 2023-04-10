use super::*;

use test_log::test;

use semver::{Version, VersionReq};
use uuid::Uuid;

#[test]
fn basic_ok_response() {
    let resp = ok("abc");

    assert_eq!(
        resp,
        MyResponse {
            code: StatusCode::OK,
            headers: vec![(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json")
            )],
            body: "abc"
        }
    );
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
                HeaderValue::from_static("67e55044-10b1-426f-9247-bb680e5fe0c8")
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
            offset: Some((
                "1.0.1".parse().unwrap(),
                "67e55044-10b1-426f-9247-bb680e5fe0c8".into(),
            )),
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
                HeaderValue::from_static("38e5f63a-4a59-4187-a0e7-3364b2c530c3")
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

    // should only match one thing, so no offset header
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
                HeaderValue::from_static("436b4766-c31c-47ca-84fd-8ed522c49191")
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

    // should only match one thing, so no offset header
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
                HeaderValue::from_static("04b459e2-a696-4531-9e7a-ae931ed38bc4")
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
